// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use clap::Parser;
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

fn main() -> Result<ExitCode> {
    let args = Cli::parse();

    let mut command = Command::new(&args.command_line[0]);
    command.args(&args.command_line[1..]);

    // Redirect output to a file if `--output` was specified.
    if let Some(log_name) = &args.output {
        let log_out = File::create(&log_name)?;
        let log_err = log_out.try_clone()?;
        command
            .stdout(Stdio::from(log_out))
            .stderr(Stdio::from(log_err));
    }

    let out = processes::run(&mut command)?;

    // If the command failed , then print saved output on the stderr.
    if !out.status.success() {
        if let Some(log_name) = &args.output {
            let mut read_file = File::open(&log_name)?;
            std::io::copy(&mut read_file, &mut std::io::stderr())?;
        }
    }

    // Propagate the exit status of the command
    // (128 + signal if it terminated after receiving a signal).
    match out.status.code() {
        Some(code) => Ok(ExitCode::from(code as u8)),
        None => {
            let signal = out
                .status
                .signal()
                .expect("signal number should be present");
            Ok(ExitCode::from(128 + signal as u8))
        }
    }
}
