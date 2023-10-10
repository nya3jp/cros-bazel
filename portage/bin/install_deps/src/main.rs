// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{ensure, Result};
use clap::Parser;
use cliutil::cli_main;
use container::{enter_mount_namespace, BindMount, CommonArgs, ContainerSettings, InstallGroup};
use durabletree::DurableTree;
use fileutil::{resolve_symlink_forest, SafeTempDirBuilder};
use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

const MAIN_SCRIPT: &str = "/mnt/host/.install_deps/install_deps.sh";

#[derive(Parser, Debug)]
#[clap()]
struct Cli {
    #[command(flatten)]
    common: CommonArgs,

    /// Name of board
    #[arg(long)]
    board: Option<String>,

    #[arg(long)]
    install_target: Vec<InstallGroup>,

    /// A path to a directory where the output durable tree is written.
    #[arg(long, required = true)]
    output: PathBuf,
}

fn do_main() -> Result<()> {
    let args = Cli::try_parse()?;

    let mutable_base_dir = SafeTempDirBuilder::new().base_dir(&args.output).build()?;

    let mut settings = ContainerSettings::new();
    settings.set_mutable_base_dir(mutable_base_dir.path());
    settings.apply_common_args(&args.common)?;

    let runfiles = runfiles::Runfiles::create()?;

    settings.push_bind_mount(BindMount {
        source: resolve_symlink_forest(
            &runfiles.rlocation("cros/bazel/portage/bin/install_deps/install_deps.sh"),
        )?,
        mount_path: PathBuf::from(MAIN_SCRIPT),
        rw: false,
    });

    let portage_pkg_dir = match &args.board {
        Some(board) => Path::new("/build").join(board).join("packages"),
        None => PathBuf::from("/var/lib/portage/pkgs"),
    };

    let (mounts, envs) = InstallGroup::get_mounts_and_env(&args.install_target, portage_pkg_dir)?;
    for mount in mounts {
        settings.push_bind_mount(mount);
    }

    let mut container = settings.prepare()?;

    let mut command = container.command(MAIN_SCRIPT);
    command.envs(envs);
    if let Some(board) = &args.board {
        command.env("BOARD", board);
    }

    let status = command.status()?;
    ensure!(status.success(), "Command failed: {:?}", status);

    // Move the upper directory contents to the output directory.
    fileutil::move_dir_contents(&container.into_upper_dir(), &args.output)?;

    // Delete the mutable base directory that contains the upper directory.
    drop(mutable_base_dir);

    container::clean_layer(&args.output)?;
    DurableTree::convert(&args.output)?;

    Ok(())
}

fn main() -> ExitCode {
    enter_mount_namespace().expect("Failed to enter a mount namespace");
    cli_main(do_main, Default::default())
}
