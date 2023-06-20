// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{ensure, Context, Result};
use clap::Parser;
use cliutil::cli_main;
use container::{enter_mount_namespace, BindMount, CommonArgs, ContainerSettings};
use durabletree::DurableTree;
use fileutil::{resolve_symlink_forest, SafeTempDirBuilder};
use std::{path::PathBuf, process::ExitCode};

const MAIN_SCRIPT: &str = "/mnt/host/.sdk_install_glibc/setup.sh";
const GLIBC_BINPKG: &str = "/mnt/host/.sdk_install_glibc/glibc.tbz2";

#[derive(Parser, Debug)]
#[clap()]
struct Cli {
    #[command(flatten)]
    common: CommonArgs,

    /// A path to a directory where the output durable tree is written.
    #[arg(long, required = true)]
    output: PathBuf,

    /// Name of board
    #[arg(long)]
    board: String,

    /// cross-*-glibc package to install into the board's sysroot
    #[arg(long)]
    glibc: PathBuf,
}

fn do_main() -> Result<()> {
    let args = Cli::parse();

    let mutable_base_dir = SafeTempDirBuilder::new().base_dir(&args.output).build()?;

    let mut settings = ContainerSettings::new();
    settings.set_mutable_base_dir(mutable_base_dir.path());
    settings.apply_common_args(&args.common)?;

    let runfiles = runfiles::Runfiles::create()?;

    let glibc_binpkg = resolve_symlink_forest(&args.glibc)?;
    settings.push_bind_mount(BindMount {
        source: glibc_binpkg,
        mount_path: GLIBC_BINPKG.into(),
        rw: false,
    });

    settings.push_bind_mount(BindMount {
        source: resolve_symlink_forest(
            &runfiles.rlocation("cros/bazel/ebuild/private/cmd/sdk_install_glibc/setup.sh"),
        )?,
        mount_path: PathBuf::from(MAIN_SCRIPT),
        rw: false,
    });

    let mut container = settings.prepare()?;

    let mut command = container.command(MAIN_SCRIPT);
    command.env("BOARD", args.board);

    let status = command.status()?;
    ensure!(status.success(), "Command failed: {:?}", status);

    // Move the upper directory contents to the output directory.
    fileutil::move_dir_contents(&container.into_upper_dir(), &args.output)
        .with_context(|| "Failed to move the upper dir.")?;

    // Delete the mutable base directory that contains the upper directory.
    drop(mutable_base_dir);

    container::clean_layer(None, &args.output)
        .with_context(|| "Failed to clean the output dir.")?;

    DurableTree::convert(&args.output)?;

    Ok(())
}

fn main() -> ExitCode {
    enter_mount_namespace().expect("Failed to enter a mount namespace");
    cli_main(do_main)
}
