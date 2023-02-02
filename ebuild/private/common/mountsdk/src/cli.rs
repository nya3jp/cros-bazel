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

    #[arg(long)]
    ebuild_log: Option<PathBuf>,

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
        for sdk in args.sdk {
            new_overlays.push(OverlayInfo {
                mount_dir: PathBuf::from("/"),
                image_path: sdk,
            });
        }
        new_overlays.extend(args.overlay);
        return Ok(Config {
            overlays: new_overlays,
            login_mode: args.login_mode,
            remounts: Vec::new(),
            cmd_prefix: vec![],
            bind_mounts: Vec::new(),
            log_file: args.ebuild_log,
        });
    }
}
