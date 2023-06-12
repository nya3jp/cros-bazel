// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{ensure, Context, Result};
use binarypackage::BinaryPackage;
use clap::Parser;
use cliutil::cli_main;
use container::{enter_mount_namespace, BindMount, CommonArgs, ContainerSettings};
use durabletree::DurableTree;
use fileutil::{resolve_symlink_forest, SafeTempDirBuilder};
use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

const BINARY_EXT: &str = ".tbz2";
const MAIN_SCRIPT: &str = "/mnt/host/.sdk_update/setup.sh";

#[derive(Parser, Debug)]
#[clap()]
struct Cli {
    #[command(flatten)]
    common: CommonArgs,

    /// A path to a directory where the output durable tree is written.
    #[arg(long, required = true)]
    output: PathBuf,

    #[arg(long)]
    install_host: Vec<PathBuf>,

    #[arg(long)]
    install_tarball: Vec<PathBuf>,
}

fn bind_binary_packages(
    settings: &mut ContainerSettings,
    packages_dir: &Path,
    package_paths: Vec<PathBuf>,
) -> Result<Vec<String>> {
    package_paths
        .into_iter()
        .map(|package_path| {
            let package_path = resolve_symlink_forest(&package_path)?;
            let bp = BinaryPackage::open(&package_path)?;
            let category_pf = bp.category_pf();
            let mount_path = packages_dir.join(format!("{category_pf}{BINARY_EXT}"));
            settings.push_bind_mount(BindMount {
                source: package_path,
                mount_path,
                rw: false,
            });
            Ok(format!("={category_pf}"))
        })
        .collect()
}

fn do_main() -> Result<()> {
    let args = Cli::parse();

    let mutable_base_dir = SafeTempDirBuilder::new().base_dir(&args.output).build()?;

    let mut settings = ContainerSettings::new();
    settings.set_mutable_base_dir(mutable_base_dir.path());
    settings.apply_common_args(&args.common)?;

    let runfiles = runfiles::Runfiles::create()?;

    let tarballs_dir = Path::new("/stage/tarballs");
    let host_packages_dir = Path::new("/var/lib/portage/pkgs");

    let host_install_atoms =
        bind_binary_packages(&mut settings, host_packages_dir, args.install_host)
            .with_context(|| "Failed to bind host binary packages.")?;

    for tarball in args.install_tarball {
        let tarball = resolve_symlink_forest(&tarball)?;
        let mount_path = tarballs_dir.join(tarball.file_name().unwrap());
        settings.push_bind_mount(BindMount {
            source: tarball,
            mount_path,
            rw: false,
        });
    }

    settings.push_bind_mount(BindMount {
        source: resolve_symlink_forest(
            &runfiles.rlocation("cros/bazel/ebuild/private/cmd/sdk_update/setup.sh"),
        )?,
        mount_path: PathBuf::from(MAIN_SCRIPT),
        rw: false,
    });

    let mut container = settings.prepare()?;

    let mut command = container.command(MAIN_SCRIPT);
    command.env("INSTALL_ATOMS_HOST", host_install_atoms.join(" "));

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
