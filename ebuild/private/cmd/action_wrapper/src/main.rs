// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use clap::Parser;
use cliutil::cli_main_quiet;
use std::fs::File;
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};

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

    let status = processes::run(&mut command)?;

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
        processes::run(&mut Command::new("true"))?;
        Ok(())
    }

    #[test]
    fn runs_failed_process() -> Result<()> {
        processes::run(&mut Command::new("false"))?;
        Ok(())
    }
}
