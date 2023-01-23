// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::mountsdk::{Config, LoginMode};
use anyhow::Result;
use clap::{arg, Args};
use makechroot::OverlayInfo;
use std::path::PathBuf;

pub const SOURCE_DIR: &str = "/mnt/host/source";

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[arg(long, required = true)]
    sdk: Vec<PathBuf>,

    #[arg(
    long,
    help = "<inside path>=<squashfs file | directory | tar.*>: Mounts the file or directory at \
            the specified path. Inside path can be absolute or relative to /mnt/host/source/.",
    required = true
    )]
    overlay: Vec<OverlayInfo>,

    #[arg(
    long = "login",
    help = "logs in to the SDK before installing deps, before building, after \
        building, or after failing to build respectively.",
    default_value_t = LoginMode::Never,
    )]
    login_mode: LoginMode,
}


impl Config {
    pub fn try_from(args: ConfigArgs) -> Result<Config> {
        let mut new_overlays: Vec<OverlayInfo> = Vec::new();
        for sdk in args.sdk.iter() {
            new_overlays.push(OverlayInfo {
                mount_dir: PathBuf::from("/"),
                image_path: sdk.to_path_buf(),
            });
        }
        new_overlays.extend(args.overlay.iter().map(|overlay| OverlayInfo {
            image_path: overlay.image_path.to_path_buf(),
            mount_dir: if overlay.mount_dir.is_absolute() {
                overlay.mount_dir.to_path_buf()
            } else {
                PathBuf::from(SOURCE_DIR).join(overlay.mount_dir.to_path_buf())
            },
        }));
        return Ok(Config {
            overlays: new_overlays,
            login_mode: args.login_mode,
            remounts: Vec::new(),
            bind_mounts: Vec::new(),
            run_in_container_extra_args: Vec::new(),
        });
    }
}
