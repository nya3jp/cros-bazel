// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use clap::Parser;
use makechroot::BindMount;
use mountsdk::{InstallGroup, MountedSDK};
use std::path::PathBuf;

const MAIN_SCRIPT: &str = "/mnt/host/bazel-build/install_deps.sh";

#[derive(Parser, Debug)]
#[clap()]
struct Cli {
    #[command(flatten)]
    mountsdk_config: mountsdk::ConfigArgs,

    #[arg(long)]
    install_target: Vec<InstallGroup>,

    #[arg(long, required = true)]
    output_dir: PathBuf,

    #[arg(long, required = true)]
    output_symlink_tar: PathBuf,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut cfg = mountsdk::Config::try_from(args.mountsdk_config)?;

    let r = runfiles::Runfiles::create()?;

    cfg.bind_mounts.push(BindMount {
        source: r.rlocation("cros/bazel/ebuild/private/cmd/install_deps/install_deps.sh"),
        mount_path: PathBuf::from(MAIN_SCRIPT),
    });

    let target_packages_dir: PathBuf = ["/build", &cfg.board, "packages"].iter().collect();

    let (mut mounts, env) =
        InstallGroup::get_mounts_and_env(&args.install_target, &target_packages_dir)?;
    cfg.bind_mounts.append(&mut mounts);
    cfg.envs = env;

    let mut sdk = MountedSDK::new(cfg)?;
    // TODO: Simplify this after tg/1717983 is submitted.
    let out_dir = sdk
        .root_dir()
        .outside
        .join(target_packages_dir.strip_prefix("/")?);
    std::fs::create_dir_all(out_dir)?;
    std::fs::create_dir_all(sdk.root_dir().outside.join("var/lib/portage/pkgs"))?;

    sdk.run_cmd(&[MAIN_SCRIPT])?;

    fileutil::move_dir_contents(sdk.diff_dir().as_path(), args.output_dir.as_path())?;
    makechroot::clean_layer(Some(&sdk.board), args.output_dir.as_path())?;
    tar::move_symlinks_into_tar(args.output_dir.as_path(), args.output_symlink_tar.as_path())?;

    Ok(())
}
