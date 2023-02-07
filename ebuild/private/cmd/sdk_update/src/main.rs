// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use clap::Parser;
use makechroot::{BindMount, OverlayInfo};
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

#[derive(Parser, Debug)]
#[clap()]
struct Cli {
    /// A path to a file representing a file system layer
    #[arg(long, required = true)]
    input: Vec<PathBuf>,

    /// A path to a directory to write non-symlink files under
    #[arg(long, required = true)]
    output_dir: PathBuf,

    /// A path to write a symlink tar to
    #[arg(long, required = true)]
    output_symlink_tar: PathBuf,

    #[arg(long, required = true)]
    board: String,

    #[arg(long)]
    install_host: Vec<PathBuf>,

    #[arg(long)]
    install_target: Vec<PathBuf>,

    #[arg(long)]
    install_tarball: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let r = runfiles::Runfiles::create()?;

    let run_in_container_path =
        r.rlocation("cros/bazel/ebuild/private/cmd/run_in_container/run_in_container_rust");

    let tmp_dir = tempdir()?;

    let scratch_dir = tmp_dir.path().join("build").join(&args.board);
    let diff_dir = scratch_dir.join("diff");

    let root_dir = fileutil::DualPath {
        outside: tmp_dir.path().join("root"),
        inside: PathBuf::from("/"),
    };
    let sysroot_dir = root_dir.join("build").join(&args.board);
    let stage_dir = root_dir.join("stage");
    let tarballs_dir = stage_dir.join("tarballs");
    let host_packages_dir = root_dir.join("var/lib/portage/pkgs");
    let target_packages_dir = sysroot_dir.join("packages");

    std::fs::create_dir_all(&stage_dir.outside)?;
    std::fs::create_dir_all(&tarballs_dir.outside)?;
    std::fs::create_dir_all(&host_packages_dir.outside)?;
    std::fs::create_dir_all(&target_packages_dir.outside)?;

    let host_install_atoms =
        makechroot::copy_binary_packages(&host_packages_dir.outside, &args.install_host)
            .with_context(|| "Failed to copy host binary packages.")?;
    let target_install_atoms =
        makechroot::copy_binary_packages(&target_packages_dir.outside, &args.install_target)
            .with_context(|| "Failed to copy target binary packages.")?;

    for tarball in &args.install_tarball {
        std::fs::copy(
            tarball,
            tarballs_dir.join(tarball.file_name().unwrap()).outside,
        )
        .with_context(|| "Failed to copy tarballs.")?;
    }

    let script_path = stage_dir.join("setup.sh");
    std::fs::copy(
        r.rlocation("cros/bazel/ebuild/private/cmd/sdk_update/setup.sh"),
        script_path.outside,
    )
    .with_context(|| "Failed to copy the script.")?;

    let overlays = args
        .input
        .into_iter()
        .map(|input_path| OverlayInfo {
            mount_dir: PathBuf::from("/"),
            image_path: input_path,
        })
        .collect();

    let bind_mounts = vec![
        BindMount {
            mount_path: stage_dir.inside,
            source: stage_dir.outside,
        },
        BindMount {
            mount_path: host_packages_dir.inside,
            source: host_packages_dir.outside,
        },
        BindMount {
            mount_path: target_packages_dir.inside,
            source: target_packages_dir.outside,
        },
    ];

    let config_path = tmp_dir.path().join("run_in_container_args.json");
    run_in_container_lib::RunInContainerConfig {
        staging_dir: scratch_dir,
        chdir: PathBuf::from("/"),
        overlays,
        bind_mounts,
        keep_host_mount: false,
    }
    .serialize_to(&config_path)?;

    processes::run_and_check(
        Command::new(run_in_container_path)
            .arg("--cfg")
            .arg(&config_path)
            .arg("--cmd")
            .arg(script_path.inside)
            .envs(std::env::vars())
            .env("PATH", "/usr/sbin:/usr/bin:/sbin:/bin")
            .env("BOARD", &args.board)
            .env("INSTALL_ATOMS_HOST", host_install_atoms.join(" "))
            .env("INSTALL_ATOMS_TARGET", target_install_atoms.join(" ")),
    )
    .with_context(|| format!("Failed to execute run_in_container."))?;

    fileutil::move_dir_contents(&diff_dir, &args.output_dir)
        .with_context(|| "Failed to move the diff dir.")?;

    // Some of the folders in the overlayfs workdir have 000 permissions.
    // We need to grant rw permissions to the directories so `os.RemoveAll`
    // doesn't fail.
    fileutil::remove_dir_all_with_chmod(tmp_dir.path())
        .with_context(|| "Failed to remove the temporary directory.")?;

    makechroot::clean_layer(Some(&args.board), &args.output_dir)
        .with_context(|| "Failed to clean the output dir.")?;

    tar::move_symlinks_into_tar(&args.output_dir, &args.output_symlink_tar)
        .with_context(|| "Failed to move symlinks into a tarball.")?;

    Ok(())
}
