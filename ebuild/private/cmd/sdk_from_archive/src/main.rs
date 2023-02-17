// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use clap::Parser;
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

#[derive(Parser, Debug)]
#[clap()]
struct Cli {
    /// A path to a .tar.xz archive file containing base SDK
    #[arg(long, required = true)]
    input: PathBuf,

    /// A path to a directory to write non-symlink files under
    #[arg(long, required = true)]
    output_dir: PathBuf,

    /// A path to write a symlink tar to
    #[arg(long, required = true)]
    output_symlink_tar: PathBuf,
}

fn main() -> Result<ExitCode> {
    let args = Cli::parse();

    std::fs::create_dir_all(&args.output_dir)?;

    // TODO: Remove the dependency to the system pixz.
    let status = Command::new("/usr/bin/tar")
        .args([
            "-I/usr/bin/pixz",
            "-xf",
            &args.input.to_string_lossy(),
            "-C",
            &args.output_dir.to_string_lossy(),
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

    tar::move_symlinks_into_tar(&args.output_dir, &args.output_symlink_tar)?;

    Ok(ExitCode::SUCCESS)
}
