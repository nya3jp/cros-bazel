// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::process::Command;

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

fn main() -> Result<()> {
    let args = Cli::parse();

    std::fs::create_dir_all(&args.output_dir)?;

    // TODO: Remove the dependency to the system pixz.
    processes::run_and_check(Command::new("/usr/bin/tar").args([
        "-I/usr/bin/pixz",
        "-xf",
        &args.input.to_string_lossy(),
        "-C",
        &args.output_dir.to_string_lossy(),
    ]))?;

    tar::move_symlinks_into_tar(&args.output_dir, &args.output_symlink_tar)?;

    Ok(())
}
