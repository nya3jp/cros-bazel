// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use clap::Parser;
use cliutil::cli_main;
use container::{InstallGroup, MountedSDK};
use makechroot::BindMount;
use std::fs::File;
use std::{path::PathBuf, process::ExitCode};

const MAIN_SCRIPT: &str = "/mnt/host/bazel-build/build_sdk.sh";

#[derive(Parser, Debug)]
#[clap()]
struct Cli {
    #[command(flatten)]
    mountsdk_config: container::ConfigArgs,

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
    let mut cfg = container::MountSdkConfig::try_from(args.mountsdk_config)?;

    let r = runfiles::Runfiles::create()?;

    cfg.bind_mounts.push(BindMount {
        source: r.rlocation("cros/bazel/ebuild/private/cmd/build_sdk/build_sdk.sh"),
        mount_path: PathBuf::from(MAIN_SCRIPT),
        rw: false,
    });

    // Create the output file, then drop the reference to close the handle.
    // We need the file to exist so the bind mount will work.
    drop(File::create(&args.output)?);

    // We want the container to directly write to the output file to avoid
    // copying the tarball from /tmp to the output root.
    cfg.bind_mounts.push(BindMount {
        source: args.output,
        mount_path: PathBuf::from("/mnt/host/bazel-build/output.tar.zst"),
        rw: true,
    });

    let target_packages_dir: PathBuf = ["/build", &args.board, "packages"].iter().collect();

    let (mut mounts, env) =
        InstallGroup::get_mounts_and_env(&args.install_target, target_packages_dir)?;
    cfg.bind_mounts.append(&mut mounts);
    cfg.envs = env;

    let mut sdk = MountedSDK::new(cfg, Some(&args.board))?;

    sdk.run_cmd(&[MAIN_SCRIPT])?;

    Ok(())
}

fn main() -> ExitCode {
    cli_main(do_main)
}
