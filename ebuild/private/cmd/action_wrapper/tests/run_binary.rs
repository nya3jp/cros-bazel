// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use regex::Regex;
use std::fmt;
use std::process::{Command, ExitStatus, Stdio};
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

struct ActionWrapperOutputs {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
    pub log: String,
}

fn run_action_wrapper(termination_kind: TerminationKind) -> Result<ActionWrapperOutputs> {
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

    Ok(ActionWrapperOutputs {
        status: output.status,
        stdout: String::from_utf8(output.stdout)?,
        stderr: String::from_utf8(output.stderr)?,
        log: std::fs::read_to_string(out_file.path())?,
    })
}

const PROGRAM_NAME: &str = "action_wrapper";
const TEST_SCRIPT_OUTPUT: &str = "stdout ONE\nstderr TWO\n";

#[test]
fn redirected_error() -> Result<()> {
    let log_re = Regex::new(&format!(
        r"^{TEST_SCRIPT_OUTPUT}{PROGRAM_NAME}: Command exited with code 40 in \d+\.\d+s\n"
    ))
    .unwrap();

    let outputs = run_action_wrapper(TerminationKind::ExitCode(40))?;

    assert_eq!(outputs.status.code(), Some(40));
    assert_eq!(outputs.stdout, "");
    assert!(
        log_re.is_match(&outputs.stderr),
        "stderr: {}",
        outputs.stderr
    );
    assert!(log_re.is_match(&outputs.log), "log: {}", outputs.log);
    Ok(())
}

#[test]
fn redirected_success() -> Result<()> {
    let log_re = Regex::new(&format!(
        r"^{TEST_SCRIPT_OUTPUT}{PROGRAM_NAME}: Command exited with code 0 in \d+\.\d+s\n"
    ))
    .unwrap();

    let outputs = run_action_wrapper(TerminationKind::ExitCode(0))?;

    assert_eq!(outputs.status.code(), Some(0));
    assert_eq!(outputs.stdout, "");
    assert_eq!(outputs.stderr, "");
    assert!(log_re.is_match(&outputs.log), "log: {}", outputs.log);
    Ok(())
}

#[test]
fn redirected_signal() -> Result<()> {
    let log_re = Regex::new(&format!(
        r"^{TEST_SCRIPT_OUTPUT}{PROGRAM_NAME}: Command killed with signal SIGUSR1 in \d+\.\d+s\n"
    ))
    .unwrap();

    let outputs = run_action_wrapper(TerminationKind::Signal(String::from("USR1")))?;

    assert_eq!(outputs.status.code(), Some(128 + libc::SIGUSR1));
    assert_eq!(outputs.stdout, "");
    assert!(
        log_re.is_match(&outputs.stderr),
        "stderr: {}",
        outputs.stderr
    );
    assert!(log_re.is_match(&outputs.log), "log: {}", outputs.log);
    Ok(())
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
