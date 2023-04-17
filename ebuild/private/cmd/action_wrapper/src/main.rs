// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use clap::Parser;
use cliutil::cli_main_quiet;
use nix::sys::signal::Signal;
use signal_hook::consts::signal::{SIGCHLD, SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use std::fs::File;
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::{Command, ExitCode, ExitStatus, Stdio};

#[derive(Parser, Debug)]
#[clap(
    about = "Redirect command output to a file and also print it on error.",
    author, version, about, long_about=None, trailing_var_arg = true)]
struct Cli {
    #[arg(help = "File to save stdout and stderr to", long)]
    output: Option<PathBuf>,

    #[arg(help = "Command to run", required = true)]
    command_line: Vec<String>,
}

/// Runs a child process, with some special signal handling.
///   - Forwards SIGTERM to the child processes
///   - Ignores SIGINT while the processes is running. SIGINT is normally generated
///     by the terminal when Ctrl+C is pressed. The signal is sent to all processes
///     in the foreground processes group. This means that the child processes
///     should receive the signal by default so we don't need to forward it. One
///     exception is if the child puts itself into a different processes group, but
///     we want to avoid that.
fn run(cmd: &mut Command) -> Result<ExitStatus> {
    // Register the signal handler before spawning the process to ensure we don't drop any signals.
    let mut signals = Signals::new([SIGCHLD, SIGINT, SIGTERM])?;

    let mut child = cmd.spawn()?;

    for signal in signals.forever() {
        match signal as libc::c_int {
            SIGCHLD => match &child.try_wait()? {
                Some(_) => return Ok(child.wait()?),
                None => continue,
            },
            SIGINT => {}
            SIGTERM => nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(child.id().try_into()?),
                Signal::SIGTERM,
            )?,
            _ => unreachable!(),
        }
    }
    unreachable!()
}

fn do_main() -> Result<ExitCode> {
    let args = Cli::parse();

    // Always enable Rust backtraces.
    std::env::set_var("RUST_BACKTRACE", "1");

    let mut command = Command::new(&args.command_line[0]);
    command.args(&args.command_line[1..]);

    // Redirect output to a file if `--output` was specified.
    if let Some(log_name) = &args.output {
        let log_out = File::create(log_name)?;
        let log_err = log_out.try_clone()?;
        command
            .stdout(Stdio::from(log_out))
            .stderr(Stdio::from(log_err));
    }

    let status = run(&mut command)?;

    // If the command failed , then print saved output on the stderr.
    if !status.success() {
        if let Some(log_name) = &args.output {
            let mut read_file = File::open(log_name)?;
            std::io::copy(&mut read_file, &mut std::io::stderr())?;
        }
    }

    // Propagate the exit status of the command
    // (128 + signal if it terminated after receiving a signal).
    match status.code() {
        Some(code) => Ok(ExitCode::from(code as u8)),
        None => {
            let signal = status.signal().expect("signal number should be present");
            Ok(ExitCode::from(128 + signal as u8))
        }
    }
}

fn main() -> ExitCode {
    // We use `cli_main_quiet` instead of `cli_main` to suppress the preamble
    // logs because action_wrapper must queue stdout/stderr until it sees the
    // wrapped program to exit abnormally. This means we don't log the arguments
    // passed to action_wrapper itself, but the wrapped program should soon
    // print one with `cli_main`.
    cli_main_quiet(do_main)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_process() -> Result<()> {
        run(&mut Command::new("true"))?;
        Ok(())
    }

    #[test]
    fn runs_failed_process() -> Result<()> {
        run(&mut Command::new("false"))?;
        Ok(())
    }
}
