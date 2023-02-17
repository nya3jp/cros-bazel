// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use std::fmt;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

#[derive(PartialEq)]
enum TerminationKind {
    ExitCode(u8),
    Signal(String),
}

impl fmt::Display for TerminationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TerminationKind::ExitCode(code) => write!(f, "{code}"),
            TerminationKind::Signal(signal) => write!(f, "{signal}"),
        }
    }
}

fn base_test(termination_kind: TerminationKind, expected_code: i32) -> Result<()> {
    let out_file = NamedTempFile::new()?;

    let mut command = Command::new(env!("CARGO_BIN_EXE_action_wrapper"));
    command
        .arg("--output")
        .arg(out_file.path())
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/test_script.sh"))
        .arg(format!("{termination_kind}"))
        .arg("ONE")
        .arg("TWO");

    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    assert_eq!(output.status.code(), Some(expected_code));

    let actual_printed_stdout = String::from_utf8(output.stdout)?;
    let actual_printed_stderr = String::from_utf8(output.stderr)?;

    if termination_kind == TerminationKind::ExitCode(0) {
        assert_eq!(actual_printed_stdout, "");
        assert_eq!(actual_printed_stderr, "");
    } else {
        assert_eq!(actual_printed_stdout, "");
        assert_eq!(actual_printed_stderr, "stdout ONE\nstderr TWO\n");
    }

    let actual_saved_output = std::fs::read_to_string(out_file.path())?;
    assert_eq!(actual_saved_output, "stdout ONE\nstderr TWO\n");

    Ok(())
}

#[test]
fn redirected_error() -> Result<()> {
    base_test(TerminationKind::ExitCode(40), 40)
}

#[test]
fn redirected_success() -> Result<()> {
    base_test(TerminationKind::ExitCode(0), 0)
}

#[test]
fn redirected_signal() -> Result<()> {
    base_test(
        TerminationKind::Signal(String::from("USR1")),
        128 + libc::SIGUSR1,
    )
}

#[test]
fn run_without_arguments() -> Result<()> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_action_wrapper"));

    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    assert!(!output.status.success());

    let actual_printed_stderr = String::from_utf8(output.stderr)?;

    // We only check a part of a long error message.
    assert!(
        actual_printed_stderr.contains("required arguments were not provided"),
        "unexpected stderr: {}",
        actual_printed_stderr
    );
    Ok(())
}
