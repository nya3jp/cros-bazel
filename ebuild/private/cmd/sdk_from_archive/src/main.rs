// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use clap::Parser;
use cliutil::cli_main;
use durabletree::DurableTree;
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

#[derive(Parser, Debug)]
#[clap()]
struct Cli {
    /// A path to a .tar.xz archive file containing base SDK.
    #[arg(long, required = true)]
    input: PathBuf,

    /// A path to a directory where the output durable tree is written.
    #[arg(long, required = true)]
    output: PathBuf,
}

fn do_main() -> Result<ExitCode> {
    let args = Cli::parse();

    std::fs::create_dir_all(&args.output)?;

    // TODO: Remove the dependency to the system pixz.
    let status = Command::new("tar")
        .args([
            "-Ipixz",
            "-xf",
            &args.input.to_string_lossy(),
            "-C",
            &args.output.to_string_lossy(),
        ])
        .status()?;

    if !status.success() {
        // Propagate the exit status of the command
        // (128 + signal if it terminated after receiving a signal).
        let tar_code = status
            .code()
            .unwrap_or_else(|| 128 + status.signal().expect("signal number should be present"));
        return Ok(ExitCode::from(tar_code as u8));
    }

    DurableTree::convert(&args.output)?;

    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    cli_main(do_main)
}
