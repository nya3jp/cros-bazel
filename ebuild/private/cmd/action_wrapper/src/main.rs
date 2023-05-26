// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use clap::Parser;
use cliutil::{handle_top_level_result, PROFILES_DIR_ENV};
use processes::status_to_exit_code;
use std::fs::File;
use std::io::Write;
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

const PROGRAM_NAME: &str = "action_wrapper";

#[derive(Parser, Debug)]
#[clap(
    about = "Redirect command output to a file and also print it on error.",
    author, version, about, long_about=None, trailing_var_arg = true)]
struct Cli {
    #[arg(help = "File to save stdout and stderr to", long)]
    log: Option<PathBuf>,

    #[arg(help = "Directory to save profile JSON files to", long)]
    profiles: Option<PathBuf>,

    #[arg(help = "Command to run", required = true)]
    command_line: Vec<String>,
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

    let mut command = Command::new(&args.command_line[0]);
    command.args(&args.command_line[1..]);

    if let Some(output) = &output {
        command
            .stdout(Stdio::from(output.try_clone()?))
            .stderr(Stdio::from(output.try_clone()?));
    }

    if let Some(profiles_dir) = &args.profiles {
        command.env(PROFILES_DIR_ENV, profiles_dir);
    }

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

    // If the command failed, then print saved output on the stderr.
    if !status.success() {
        if let Some(log_name) = &args.log {
            let mut read_file = File::open(log_name)?;
            std::io::copy(&mut read_file, &mut std::io::stderr())?;
        }
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
