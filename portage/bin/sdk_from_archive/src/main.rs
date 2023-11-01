// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::bail;
use anyhow::Result;
use clap::Parser;
use cliutil::cli_main;
use durabletree::DurableTree;
use runfiles::Runfiles;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[clap()]
struct Cli {
    /// A path to a .tar.{xz,zst} archive file containing base SDK.
    #[arg(long, required = true)]
    input: PathBuf,

    /// A path to a directory where the output durable tree is written.
    #[arg(long, required = true)]
    output: PathBuf,
}

fn do_main() -> Result<()> {
    let r = Runfiles::create()?;

    let args = Cli::try_parse()?;

    std::fs::create_dir_all(&args.output)?;

    let mut command = Command::new("tar");

    if let Some(ext) = args.input.extension() {
        // TODO: Remove the dependency to the system pixz.
        if ext == OsStr::new("xz") {
            command.arg("-Ipixz");
        } else if ext == OsStr::new("zst") {
            command.arg(format!("-I{}", r.rlocation("zstd/zstd").display()));
        } else {
            bail!("Unsupported extension: {:?}", ext);
        }
    } else {
        bail!("Input missing extension: {}", args.input.display());
    }

    command.args([
        "-xf",
        &args.input.to_string_lossy(),
        "--exclude=./etc/make.conf",
        "--exclude=./etc/make.conf.*",
        "--exclude=./etc/portage",
        "-C",
        &args.output.to_string_lossy(),
    ]);

    processes::run_and_check(&mut command)?;

    DurableTree::convert(&args.output)?;

    Ok(())
}

fn main() -> ExitCode {
    cli_main(do_main, Default::default())
}
