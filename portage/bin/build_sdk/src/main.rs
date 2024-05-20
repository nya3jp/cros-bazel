// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{ensure, Context, Result};
use clap::Parser;
use cliutil::cli_main;
use container::{enter_mount_namespace, BindMount, CommonArgs, ContainerSettings};
use durabletree::DurableTree;
use fileutil::resolve_symlink_forest;

use std::{path::PathBuf, process::ExitCode};

const MAIN_SCRIPT: &str = "/mnt/host/.build_sdk/build_sdk.sh";

#[derive(Parser, Debug)]
#[clap()]
struct Cli {
    #[command(flatten)]
    common: CommonArgs,

    /// Name of board
    #[arg(long, required = true)]
    board: String,

    /// The path of output file.
    /// A .tar.zst suffix is expected
    #[arg(long, required = true)]
    output: PathBuf,
}

fn do_main() -> Result<()> {
    let args = Cli::try_parse()?;

    let mut settings = ContainerSettings::new();
    settings.apply_common_args(&args.common)?;

    let r = runfiles::Runfiles::create()?;

    settings.push_bind_mount(BindMount {
        source: resolve_symlink_forest(&runfiles::rlocation!(
            r,
            "cros/bazel/portage/bin/build_sdk/build_sdk.sh"
        ))?,
        mount_path: PathBuf::from(MAIN_SCRIPT),
        rw: false,
    });

    fileutil::remove_dir_all_with_chmod(&args.output)
        .with_context(|| format!("rm -r {:?}", args.output))?;

    std::fs::create_dir_all(&args.output).with_context(|| format!("mkdir -p {:?}", args.output))?;

    // We want the container to directly write to the output file to avoid
    // copying the tarball from /tmp to the output root.
    settings.push_bind_mount(BindMount {
        source: args.output.clone(),
        mount_path: PathBuf::from("/mnt/host/.build_sdk/output"),
        rw: true,
    });

    let mut container = settings.prepare()?;

    let mut command = container.command(MAIN_SCRIPT);
    command.env("BOARD", &args.board);

    let status = command.status()?;
    ensure!(status.success());

    DurableTree::convert(&args.output)?;

    Ok(())
}

fn main() -> ExitCode {
    enter_mount_namespace().expect("Failed to enter a mount namespace");
    cli_main(do_main, Default::default())
}
