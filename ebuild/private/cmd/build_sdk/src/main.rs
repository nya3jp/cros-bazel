// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{ensure, Result};
use clap::Parser;
use cliutil::cli_main;
use container::{enter_mount_namespace, BindMount, CommonArgs, ContainerSettings, InstallGroup};
use fileutil::resolve_symlink_forest;
use std::fs::File;
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

    #[arg(long)]
    install_target: Vec<InstallGroup>,

    /// The path of output file.
    /// A .tar.zst suffix is expected
    #[arg(long, required = true)]
    output: PathBuf,
}

fn do_main() -> Result<()> {
    let args = Cli::parse();

    let mut settings = ContainerSettings::new();
    settings.apply_common_args(&args.common)?;

    let runfiles = runfiles::Runfiles::create()?;

    settings.push_bind_mount(BindMount {
        source: resolve_symlink_forest(
            &runfiles.rlocation("cros/bazel/ebuild/private/cmd/build_sdk/build_sdk.sh"),
        )?,
        mount_path: PathBuf::from(MAIN_SCRIPT),
        rw: false,
    });

    // Create the output file, then drop the reference to close the handle.
    // We need the file to exist so the bind mount will work.
    drop(File::create(&args.output)?);

    // We want the container to directly write to the output file to avoid
    // copying the tarball from /tmp to the output root.
    settings.push_bind_mount(BindMount {
        source: args.output,
        mount_path: PathBuf::from("/mnt/host/.build_sdk/output.tar.zst"),
        rw: true,
    });

    let target_packages_dir: PathBuf = ["/build", &args.board, "packages"].iter().collect();

    let (mounts, envs) =
        InstallGroup::get_mounts_and_env(&args.install_target, target_packages_dir)?;
    for mount in mounts {
        settings.push_bind_mount(mount);
    }

    let mut container = settings.prepare()?;

    let mut command = container.command(MAIN_SCRIPT);
    command.env("BOARD", &args.board);
    command.envs(envs);

    let status = command.status()?;
    ensure!(status.success());

    Ok(())
}

fn main() -> ExitCode {
    enter_mount_namespace().expect("Failed to enter a mount namespace");
    cli_main(do_main)
}
