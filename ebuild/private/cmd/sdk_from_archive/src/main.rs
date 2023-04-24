// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use clap::Parser;
use cliutil::cli_main;
use durabletree::DurableTree;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;

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

fn do_main() -> Result<()> {
    let args = Cli::parse();

    std::fs::create_dir_all(&args.output)?;

    // TODO: Remove the dependency to the system pixz.
    processes::run_and_check(Command::new("tar").args([
        "-Ipixz",
        "-xf",
        &args.input.to_string_lossy(),
        "-C",
        &args.output.to_string_lossy(),
    ]))?;

    DurableTree::convert(&args.output)?;

    Ok(())
}

fn main() -> ExitCode {
    cli_main(do_main)
}
