// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Provides functions common to all Rust-based CLI programs.

use itertools::Itertools;
use std::{
    ffi::OsStr,
    process::{ExitCode, Termination},
};

use anyhow::{bail, Result};

mod config;
mod logging;
mod param_file;
mod stdio_redirector;

pub use crate::config::*;
pub use crate::logging::*;
pub use crate::param_file::expanded_args_os;
pub use crate::stdio_redirector::{RedirectorConfig, StdioRedirector};

/// Wraps a CLI main function to provide the common startup/cleanup logic.
///
/// Most programs likely want to call this function at the very beginning of main.
/// Exceptions include:
/// - Programs that want to stay single-threaded (e.g. run_in_container that
///   calls unshare(2)).
pub fn cli_main<F, T>(main: F, config: Config) -> ExitCode
where
    F: FnOnce() -> Result<T, anyhow::Error> + std::panic::UnwindSafe,
    T: Termination,
{
    let _log_guard = config.logging.setup().unwrap();
    if config.log_command_line {
        log_current_command_line();
    }

    let result = match std::panic::catch_unwind(main) {
        Ok(result) => result,
        Err(err) => {
            if let Some(redirector) = config.stdio_redirector {
                redirector.flush_to_real_stderr().unwrap();
            }
            std::panic::resume_unwind(err);
        }
    };

    // ExitCode doesn't implement Eq, so we can't check whether it's a success or a failure.
    let failure = result.is_err();

    let exit_code = handle_top_level_result(result);

    if failure {
        if let Some(redirector) = config.stdio_redirector {
            redirector.flush_to_real_stderr().unwrap();
        }
    }

    exit_code
}

/// Logs the command line of the current process.
///
/// You don't need this function if you use [`cli_main`] because it calls this
/// function for you.
pub fn log_current_command_line() {
    let escaped_command = std::env::args()
        .map(|s| shell_escape::escape(s.into()))
        .join(" ");
    tracing::info!("COMMAND: {}", escaped_command);
}

/// Handles the top-level [`Result`] and returns [`ExitCode`] to be returned.
///
/// You don't need this function if you use [`cli_main`].
pub fn handle_top_level_result<T: Termination>(result: Result<T, anyhow::Error>) -> ExitCode {
    match result {
        Err(error) => {
            if let Some(error) = error.downcast_ref::<clap::Error>() {
                let _ = error.print();
                // TODO: Switch to the following line once we upgrade clap. The newer clap version
                // requires a new version of rust, which is a nontrivial upgrade.
                // ExitCode::from(error.exit_code() as u8)
                // This is what clap's error.exit_code() will return.
                // https://github.com/clap-rs/clap/blob/29f22c193c484031f1444cca4b0ed41315db82a1/clap_builder/src/util/mod.rs#L30
                ExitCode::from(2)
            } else {
                eprintln!("FATAL: {}: {:?}", get_current_process_name(), error);
                ExitCode::FAILURE
            }
        }
        Ok(value) => value.report(),
    }
}

/// Returns the current process name, or `__unknown__` if it failed to get one.
fn get_current_process_name() -> String {
    let current_exe = std::env::current_exe().unwrap_or_default();
    current_exe
        .file_name()
        .unwrap_or(OsStr::new("__unknown__"))
        .to_string_lossy()
        .into_owned()
}

// DEPRECATED: This function was put here just because several executables had
// similar logic, but it's a too small function to share here.
pub fn split_key_value(spec: &str) -> Result<(&str, &str)> {
    let v: Vec<_> = spec.split('=').collect();
    if v.len() != 2 {
        bail!("invalid spec: {:?}", spec);
    }
    Ok((v[0], v[1]))
}
