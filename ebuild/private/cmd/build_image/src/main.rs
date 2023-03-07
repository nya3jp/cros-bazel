// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use binarypackage::BinaryPackage;
use clap::Parser;
use makechroot::BindMount;
use mountsdk::MountedSDK;
use std::path::{Path, PathBuf};

const MAIN_SCRIPT: &str = "/mnt/host/bazel-build/build_image.sh";

#[derive(Parser, Debug)]
#[clap()]
pub struct Cli {
    #[command(flatten)]
    mountsdk_config: mountsdk::ConfigArgs,

    /// Output file path.
    #[arg(long)]
    output: PathBuf,

    /// File paths to binary packages to be installed on the output image.
    #[arg(long)]
    target_package: Vec<PathBuf>,

    /// File paths to host binary packages to be made available to the
    /// build_image script.
    #[arg(long)]
    host_package: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let r = runfiles::Runfiles::create()?;

    let mut cfg = mountsdk::Config::try_from(args.mountsdk_config)?;
    cfg.privileged = true;

    cfg.bind_mounts.push(BindMount {
        source: r
            .rlocation("cros/bazel/ebuild/private/cmd/build_image/container_files/edb_chromeos"),
        mount_path: Path::new("/build")
            .join(&cfg.board)
            .join("var/cache/edb/chromeos"),
    });
    cfg.bind_mounts.push(BindMount {
        source: r.rlocation(
            "cros/bazel/ebuild/private/cmd/build_image/container_files/package.provided",
        ),
        mount_path: Path::new("/build")
            .join(&cfg.board)
            .join("etc/portage/profile/package.provided"),
    });
    cfg.bind_mounts.push(BindMount {
        source: r
            .rlocation("cros/bazel/ebuild/private/cmd/build_image/container_files/build_image.sh"),
        mount_path: PathBuf::from(MAIN_SCRIPT),
    });

    for path in args.target_package {
        let package = BinaryPackage::open(&path)?;
        let mount_path = Path::new("/build")
            .join(&cfg.board)
            .join("packages")
            .join(format!("{}.tbz2", package.category_pf()));
        cfg.bind_mounts.push(BindMount {
            mount_path,
            source: path,
        });
    }

    for path in args.host_package {
        let package = BinaryPackage::open(&path)?;
        let mount_path =
            Path::new("/var/lib/portage/pkgs").join(format!("{}.tbz2", package.category_pf()));
        cfg.bind_mounts.push(BindMount {
            mount_path,
            source: path,
        });
    }

    cfg.envs.insert("BOARD".to_owned(), cfg.board.clone());
    cfg.envs
        .insert("HOST_UID".to_owned(), users::get_current_uid().to_string());
    cfg.envs
        .insert("HOST_GID".to_owned(), users::get_current_gid().to_string());

    let mut sdk = MountedSDK::new(cfg)?;
    sdk.run_cmd(&[
        MAIN_SCRIPT,
        &format!("--board={}", &sdk.board),
        // TODO: at some point, we should support a variety of image types
        "base",
        // TODO: add unparsed command-line args.
    ])?;

    let path = Path::new("mnt/host/source/src/build/images")
        .join(&sdk.board)
        .join("latest/chromiumos_base_image.bin");
    std::fs::copy(sdk.diff_dir().join(path), args.output)?;

    Ok(())
}
