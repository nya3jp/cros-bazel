// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use clap::Parser;
use cliutil::cli_main;
use durabletree::DurableTree;
use makechroot::BindMount;
use mountsdk::{InstallGroup, MountedSDK};
use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

const MAIN_SCRIPT: &str = "/mnt/host/bazel-build/install_deps.sh";

#[derive(Parser, Debug)]
#[clap()]
struct Cli {
    #[command(flatten)]
    mountsdk_config: mountsdk::ConfigArgs,

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
    let args = Cli::parse();
    let mut cfg = mountsdk::Config::try_from(args.mountsdk_config)?;

    let r = runfiles::Runfiles::create()?;

    cfg.bind_mounts.push(BindMount {
        source: r.rlocation("cros/bazel/ebuild/private/cmd/install_deps/install_deps.sh"),
        mount_path: PathBuf::from(MAIN_SCRIPT),
        rw: false,
    });

    let portage_pkg_dir = match &args.board {
        Some(board) => Path::new("/build").join(board).join("packages"),
        None => PathBuf::from("/var/lib/portage/pkgs"),
    };

    let (mut mounts, env) =
        InstallGroup::get_mounts_and_env(&args.install_target, portage_pkg_dir)?;
    cfg.bind_mounts.append(&mut mounts);
    cfg.envs = env;

    let mut sdk = MountedSDK::new(cfg, args.board.as_deref())?;

    sdk.run_cmd(&[MAIN_SCRIPT])?;

    fileutil::move_dir_contents(sdk.diff_dir().as_path(), &args.output)?;
    makechroot::clean_layer(args.board.as_deref(), &args.output)?;
    DurableTree::convert(&args.output)?;

    Ok(())
}

fn main() -> ExitCode {
    cli_main(do_main)
}
