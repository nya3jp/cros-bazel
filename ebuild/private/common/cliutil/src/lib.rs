// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Provides functions common to all Rust-based CLI programs.

use std::{
    ffi::OsStr,
    fmt::Debug,
    process::{ExitCode, Termination},
};

use anyhow::{bail, Result};
use itertools::Itertools;

/// Wraps a CLI main function to provide the common startup/cleanup logic.
pub fn cli_main<F, T, E>(main: F) -> ExitCode
where
    F: FnOnce() -> Result<T, E>,
    T: Termination,
    E: Debug,
{
    // Print the original command line.
    let escaped_command = std::env::args()
        .map(|s| shell_escape::escape(s.into()))
        .join(" ");
    eprintln!("COMMAND: {}", escaped_command);

    cli_main_quiet(main)
}

/// Similar to [`cli_main`] but it doesn't write the preamble message to stderr.
///
/// The only valid caller of this function is action_wrapper that should not
/// write to stderr. Other programs should be wrapped with action_wrapper and
/// call [`cli_main`] instead.
pub fn cli_main_quiet<F, T, E>(main: F) -> ExitCode
where
    F: FnOnce() -> Result<T, E>,
    T: Termination,
    E: Debug,
{
    let result = main();

    // Print the error and exit.
    match result {
        Err(error) => {
            let current_exe = std::env::current_exe().unwrap_or_default();
            let current_exe_name = current_exe
                .file_name()
                .unwrap_or(OsStr::new("<unknown>"))
                .to_string_lossy();

            eprintln!("FATAL: {}: {:?}", current_exe_name, error);
            ExitCode::FAILURE
        }
        Ok(value) => value.report(),
    }
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
