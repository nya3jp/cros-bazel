// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use clap::Parser;
use std::fs::File;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

#[derive(Parser, Debug)]
#[clap(
    about = "Redirect command output to files and also print it on error.",
    author, version, about, long_about=None, trailing_var_arg = true)]
struct Cli {
    #[arg(help = "File to save the standard output to", long)]
    stdout: Option<PathBuf>,

    #[arg(help = "File to save the standard error to", long)]
    stderr: Option<PathBuf>,

    #[arg(help = "Command to run", required = true)]
    command_line: Vec<String>,
}

fn redirect_output(log_name: &Path, cb: impl FnOnce(Stdio)) -> Result<()> {
    let log_file = File::create(&log_name)?;
    cb(Stdio::from(log_file));
    Ok(())
}

fn show_saved<W>(log_name: &Path, writer: &mut W) -> Result<()>
where
    W: std::io::Write,
{
    let mut read_file = File::open(&log_name)?;
    std::io::copy(&mut read_file, writer)?;
    Ok(())
}

fn main() -> Result<ExitCode> {
    let args = Cli::parse();

    let mut command = Command::new(&args.command_line[0]);
    command.args(&args.command_line[1..]);

    // TODO(ttylenda): handle redirecting stderr and stdout to the same file.

    if let Some(log_name) = &args.stdout {
        redirect_output(&log_name, |stdio| {
            command.stdout(stdio);
        })?;
    }

    if let Some(log_name) = &args.stderr {
        redirect_output(&log_name, |stdio| {
            command.stderr(stdio);
        })?;
    }

    let out = processes::run(&mut command)?;

    if !out.status.success() {
        if let Some(log_name) = &args.stderr {
            show_saved(&log_name, &mut std::io::stderr())?;
        }
        if let Some(log_name) = &args.stdout {
            show_saved(&log_name, &mut std::io::stdout())?;
        }
    }

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
