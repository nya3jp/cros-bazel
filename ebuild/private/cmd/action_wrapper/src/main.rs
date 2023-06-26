// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{ensure, Context, Result};
use chrome_trace::{Event, Phase, Trace};
use clap::Parser;
use cliutil::{handle_top_level_result, PROFILES_DIR_ENV};
use fileutil::SafeTempDir;
use nix::unistd::{getgid, getuid};
use processes::status_to_exit_code;
use serde_json::json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

const PROGRAM_NAME: &str = "action_wrapper";

const SUDO_PATH: &str = "/usr/bin/sudo";

#[derive(Parser, Debug)]
#[clap(
    about = "General-purpose wrapper of programs implementing Bazel actions.",
    author, version, about, long_about=None, trailing_var_arg = true)]
struct Cli {
    #[arg(
        help = "If set, redirects stdout/stderr of the wrapped process to \
            the specified file, and print it to stderr only when it exits \
            abnormally.",
        long
    )]
    log: Option<PathBuf>,

    #[arg(
        help = "If set, sets up environment variables of the wrapped process \
            so that it and its subprocesses records tracing data, and collects \
            them into a single Chrome tracing JSON file at the specified path.",
        long
    )]
    profile: Option<PathBuf>,

    #[arg(
        help = "Runs the wrapped process with privilege using sudo. This \
            assumes that we can run sudo without password, so typically this \
            option works only within legacy CrOS chroot. Use this option only \
            for temporary workaround during Alchemy development.",
        long
    )]
    privileged: bool,

    #[arg(
        help = "Specifies output files of the wrapped process. It can be \
            repeated multiple times. These files will be processed with \
            `sudo chown` after the wrapped process finishes so that Bazel \
            can access those files.",
        long
    )]
    privileged_output: Vec<PathBuf>,

    #[arg(help = "Command line of the wrapped process.", required = true)]
    command_line: Vec<String>,
}

fn ensure_passwordless_sudo() -> Result<()> {
    let status = Command::new(SUDO_PATH)
        .args(["-n", "true"])
        .status()
        .context("Failed to run sudo")?;
    ensure!(
        status.success(),
        "Failed to run sudo without password; run \"sudo true\" and try again"
    );
    Ok(())
}

fn merge_profiles(input_profiles_dir: &Path, output_profile_file: &Path) -> Result<()> {
    let mut merged_trace = Trace::new();

    // Load all profiles and merge events into one trace.
    for entry in std::fs::read_dir(input_profiles_dir)? {
        let entry = entry?;
        let trace = Trace::load(File::open(entry.path())?)?;
        merged_trace.events.extend(trace.events);
    }

    // Compute timestamp offsets from clock_sync metadata events.
    let mut clock_offset_by_process_id: HashMap<i64, f64> = HashMap::new();
    for event in merged_trace.events.iter() {
        if event.phase != Phase::Metadata {
            continue;
        }
        if event.name != "clock_sync" {
            continue;
        }
        let args_value = match &event.args {
            Some(a) => a,
            None => {
                continue;
            }
        };
        let args_object = match args_value.as_object() {
            Some(o) => o,
            None => {
                continue;
            }
        };
        let system_time_number = match args_object.get("system_time") {
            Some(serde_json::Value::Number(n)) => n,
            _ => {
                continue;
            }
        };
        let system_time = match system_time_number.as_f64() {
            Some(f) => f,
            None => {
                continue;
            }
        };
        let offset = system_time - event.timestamp / 1000.0;
        clock_offset_by_process_id.insert(event.process_id, offset);
    }

    // Update timestamps.
    if let Some(min_clock_offset) = clock_offset_by_process_id
        .values()
        .copied()
        .reduce(f64::min)
    {
        for event in merged_trace.events.iter_mut() {
            let clock_offset = match clock_offset_by_process_id.get(&event.process_id) {
                Some(o) => *o,
                None => {
                    // Leave unadjustable entries as-is.
                    continue;
                }
            };
            event.timestamp += clock_offset - min_clock_offset;
        }
    }

    // Also add process_sort_index metadata to ensure processes are sorted in
    // the execution order.
    let mut clock_offsets: Vec<(i64, f64)> = clock_offset_by_process_id.into_iter().collect();
    clock_offsets.sort_by(|(_, a), (_, b)| a.partial_cmp(b).expect("Clock offset is NaN"));

    for (sort_index, (process_id, _)) in clock_offsets.into_iter().enumerate() {
        merged_trace.events.push(Event {
            name: "process_sort_index".to_owned(),
            category: "".to_owned(),
            phase: Phase::Metadata,
            timestamp: 0.0,
            process_id,
            thread_id: 0,
            args: Some(json!({ "sort_index": sort_index })),
        });
    }

    // Save merged traces.
    merged_trace.save(File::create(output_profile_file)?)?;

    Ok(())
}

fn do_main() -> Result<ExitCode> {
    let args = Cli::parse();

    // Always enable Rust backtraces.
    std::env::set_var("RUST_BACKTRACE", "1");

    // Redirect output to a file if `--log` was specified.
    let mut output = if let Some(log_name) = &args.log {
        Some(File::create(log_name)?)
    } else {
        None
    };

    let mut command = if args.privileged {
        ensure_passwordless_sudo()?;
        let mut command = Command::new(SUDO_PATH);
        command.arg("--preserve-env").args(&args.command_line);
        command
    } else {
        let mut command = Command::new(&args.command_line[0]);
        command.args(&args.command_line[1..]);
        command
    };

    if let Some(output) = &output {
        command
            .stdout(Stdio::from(output.try_clone()?))
            .stderr(Stdio::from(output.try_clone()?));
    }

    let profiles_dir = SafeTempDir::new()?;
    command.env(PROFILES_DIR_ENV, profiles_dir.path());

    let start_time = Instant::now();
    let status = processes::run(&mut command)?;
    let elapsed = start_time.elapsed();

    let message = if let Some(signal_num) = status.signal() {
        let signal_name = match nix::sys::signal::Signal::try_from(signal_num) {
            Ok(signal) => signal.to_string(),
            Err(_) => signal_num.to_string(),
        };
        format!(
            "{}: Command killed with signal {} in {:.1}s",
            PROGRAM_NAME,
            signal_name,
            elapsed.as_secs_f32()
        )
    } else if let Some(code) = status.code() {
        format!(
            "{}: Command exited with code {} in {:.1}s",
            PROGRAM_NAME,
            code,
            elapsed.as_secs_f32()
        )
    } else {
        unreachable!("Unexpected ExitStatus: {:?}", status);
    };

    if let Some(output) = &mut output {
        writeln!(output, "{}", message)?;
    } else {
        eprintln!("{}", message);
    }

    // Run chown on output files by a privileged process.
    if args.privileged && !args.privileged_output.is_empty() {
        let mut command = Command::new(SUDO_PATH);
        command
            .arg("chown")
            .arg(format!("{}:{}", getuid(), getgid()))
            .arg("--")
            .args(args.privileged_output);
        if let Some(output) = &output {
            command
                .stdout(Stdio::from(output.try_clone()?))
                .stderr(Stdio::from(output.try_clone()?));
        }
        processes::run(&mut command)?;
    }

    // If the command failed, then print saved output on the stderr.
    if !status.success() {
        if let Some(log_name) = &args.log {
            let mut read_file = File::open(log_name)?;
            std::io::copy(&mut read_file, &mut std::io::stderr())?;
        }
    }

    if let Some(profile_file) = args.profile {
        merge_profiles(profiles_dir.path(), &profile_file)?;
    }

    // Propagate the exit status of the command.
    Ok(status_to_exit_code(&status))
}

fn main() -> ExitCode {
    // We don't use `cli_main` to avoid emitting the preamble logs because
    // action_wrapper must queue stdout/stderr until it sees the wrapped program
    // to exit abnormally. This means we don't log the arguments passed to
    // action_wrapper itself, but the wrapped program should soon print one with
    // `cli_main`.
    let result = do_main();
    handle_top_level_result(result)
}

#[cfg(test)]
mod tests {
    use std::process::{ExitStatus, Stdio};

    use regex::Regex;
    use tempfile::NamedTempFile;

    use super::*;

    const ACTION_WRAPPER_PATH: &str = "bazel/ebuild/private/cmd/action_wrapper/action_wrapper";

    #[derive(PartialEq)]
    enum TerminationKind {
        ExitCode(u8),
        Signal(String),
    }

    impl std::fmt::Display for TerminationKind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

        let mut command = Command::new(ACTION_WRAPPER_PATH);
        command
            .arg("--log")
            .arg(out_file.path())
            .arg("bazel/ebuild/private/cmd/action_wrapper/testdata/test_script.sh")
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
        let mut command = Command::new(ACTION_WRAPPER_PATH);

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
}
