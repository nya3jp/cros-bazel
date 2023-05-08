// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use binarypackage::BinaryPackage;
use clap::Parser;
use cliutil::cli_main;
use durabletree::DurableTree;
use makechroot::BindMount;
use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

const BINARY_EXT: &str = ".tbz2";
const MAIN_SCRIPT: &str = "/mnt/host/bazel-build/setup.sh";

#[derive(Parser, Debug)]
#[clap()]
struct Cli {
    #[command(flatten)]
    mountsdk_config: mountsdk::ConfigArgs,

    /// Name of board
    #[arg(long, required = true)]
    board: String,

    /// A path to a directory where the output durable tree is written.
    #[arg(long, required = true)]
    output: PathBuf,

    #[arg(long)]
    install_host: Vec<PathBuf>,

    #[arg(long)]
    install_tarball: Vec<PathBuf>,
}

fn bind_binary_packages(
    cfg: &mut mountsdk::Config,
    packages_dir: &Path,
    package_paths: Vec<PathBuf>,
) -> Result<Vec<String>> {
    package_paths
        .into_iter()
        .map(|package_path| {
            let bp = BinaryPackage::open(&package_path)?;
            let category_pf = bp.category_pf();
            let mount_path = packages_dir.join(format!("{category_pf}{BINARY_EXT}"));
            cfg.bind_mounts.push(BindMount {
                source: package_path,
                mount_path,
            });
            Ok(format!("={category_pf}"))
        })
        .collect()
}

fn do_main() -> Result<()> {
    let args = Cli::parse();
    let mut cfg = mountsdk::Config::try_from(args.mountsdk_config)?;

    let r = runfiles::Runfiles::create()?;

    let tarballs_dir = Path::new("/stage/tarballs");
    let host_packages_dir = Path::new("/var/lib/portage/pkgs");

    let host_install_atoms = bind_binary_packages(&mut cfg, host_packages_dir, args.install_host)
        .with_context(|| "Failed to bind host binary packages.")?;
    cfg.envs.insert(
        "INSTALL_ATOMS_HOST".to_owned(),
        host_install_atoms.join(" "),
    );

    cfg.bind_mounts
        .extend(args.install_tarball.into_iter().map(|tarball| {
            let mount_path = tarballs_dir.join(tarball.file_name().unwrap());
            BindMount {
                source: tarball,
                mount_path,
            }
        }));

    cfg.bind_mounts.push(BindMount {
        source: r.rlocation("cros/bazel/ebuild/private/cmd/sdk_update/setup.sh"),
        mount_path: PathBuf::from(MAIN_SCRIPT),
    });

    let mut sdk = mountsdk::MountedSDK::new(cfg, Some(&args.board))?;
    sdk.run_cmd(&[MAIN_SCRIPT])
        .with_context(|| "Failed to run the command.")?;

    fileutil::move_dir_contents(sdk.diff_dir(), &args.output)
        .with_context(|| "Failed to move the diff dir.")?;

    makechroot::clean_layer(Some(&args.board), &args.output)
        .with_context(|| "Failed to clean the output dir.")?;

    DurableTree::convert(&args.output)?;

    Ok(())
}

fn main() -> ExitCode {
    cli_main(do_main)
}
