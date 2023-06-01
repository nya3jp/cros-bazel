// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Provides functions common to all Rust-based CLI programs.

use std::{
    ffi::OsStr,
    fmt::Debug,
    path::{Path, PathBuf},
    process::{ExitCode, Termination},
    time::SystemTime,
};

use anyhow::{bail, Result};
use itertools::Itertools;
use tracing::info_span;
use tracing_chrome_trace::ChromeTraceLayer;
use tracing_subscriber::prelude::*;

/// Wraps a CLI main function to provide the common startup/cleanup logic.
///
/// Most programs implementing Alchemy actions likely want to call this function
/// at the very beginning of main. Exceptions include:
/// - Programs that want to avoid printing to stderr on start (e.g.
///   action_wrapper that queues stdout/stderr).
/// - Programs that want to stay single-threaded (e.g. run_in_container that
///   calls unshare(2)).
pub fn cli_main<F, T, E>(main: F) -> ExitCode
where
    F: FnOnce() -> Result<T, E>,
    T: Termination,
    E: Debug,
{
    print_current_command_line();
    let _guard = setup_tracing_by_env();
    let result = main();
    handle_top_level_result(result)
}

/// Prints the command line of the current process.
///
/// You don't need this function if you use [`cli_main`] because it calls this
/// function for you.
pub fn print_current_command_line() {
    let escaped_command = std::env::args()
        .map(|s| shell_escape::escape(s.into()))
        .join(" ");
    eprintln!("COMMAND: {}", escaped_command);
}

/// Handles the top-level [`Result`] and returns [`ExitCode`] to be returned.
///
/// You don't need this function if you use [`cli_main`] because it calls this
/// function for you.
pub fn handle_top_level_result<T: Termination, E: Debug>(result: Result<T, E>) -> ExitCode {
    match result {
        Err(error) => {
            eprintln!("FATAL: {}: {:?}", get_current_process_name(), error);
            ExitCode::FAILURE
        }
        Ok(value) => value.report(),
    }
}

/// Name of the environment variable telling Alchemy CLIs to save profiles JSON
/// under the specified directory.
pub const PROFILES_DIR_ENV: &str = "ALCHEMY_PROFILES_DIR";

/// A guard object returned by [`setup_tracing`] and [`setup_tracing_by_env`] to
/// perform cleanups with RAII.
pub struct TracingGuard {
    _span_guard: Option<tracing::span::EnteredSpan>,
    _flush_guard: Option<tracing_chrome_trace::FlushGuard>,
}

/// Sets up the standard tracing subscriber to write to the specified path, and
/// starts a span named "main".
///
/// You don't need this function if you call [`cli_main`] because they call this
/// function automatically. In other cases, call this function soon after
/// parsing command line arguments to start capturing traces.
///
/// It returns [`TracingGuard`] that performs cleanups on drop. You have to drop
/// it just before the program ends.
pub fn setup_tracing(output: &Path) -> TracingGuard {
    let (chrome_layer, flush_guard) = match ChromeTraceLayer::new(output) {
        Ok((chrome_layer, flush_guard)) => (chrome_layer, flush_guard),
        Err(err) => {
            eprintln!("WARNING: Failed to set up tracing: {:?}", err);
            return TracingGuard {
                _span_guard: None,
                _flush_guard: None,
            };
        }
    };

    tracing_subscriber::registry().with(chrome_layer).init();

    let args = std::env::args()
        .map(|s| shell_escape::escape(s.into()))
        .join(" ");
    let env = std::env::vars()
        .map(|(key, value)| format!("{}={}", key, value))
        .join("\n");
    let span_guard = info_span!("main", args = args, env = env).entered();

    TracingGuard {
        _span_guard: Some(span_guard),
        _flush_guard: Some(flush_guard),
    }
}

/// Similar to [`setup_tracing`], but derives the output path from environment
/// variables.
///
/// See [`setup_tracing`] for details.
pub fn setup_tracing_by_env() -> Option<TracingGuard> {
    if let Some(profiles_dir) = std::env::var_os(PROFILES_DIR_ENV) {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let profiles_name = format!("{}.{}.json", get_current_process_name(), timestamp);
        let profiles_path = PathBuf::from(profiles_dir).join(profiles_name);
        Some(setup_tracing(&profiles_path))
    } else {
        None
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

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn setup_tracing_works() -> Result<()> {
        const MESSAGE: &str = "hello world";

        let file = NamedTempFile::new()?;
        {
            let _guard = setup_tracing(file.path());
            tracing::info!(MESSAGE);
        }

        let content = std::fs::read_to_string(file.path())?;
        assert!(content.contains(MESSAGE));

        Ok(())
    }
}
