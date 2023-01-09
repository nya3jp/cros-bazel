// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::mountsdk::{Config, LoginMode};
use anyhow::Result;
use makechroot::OverlayInfo;
use std::path::PathBuf;

pub const SOURCE_DIR: &str = "/mnt/host/source";

pub fn get_mount_config(
    sdk: Vec<PathBuf>,
    overlays: Vec<OverlayInfo>,
    login_mode: LoginMode,
) -> Result<Config> {
    let mut new_overlays: Vec<OverlayInfo> = Vec::new();
    for sdk in sdk.iter() {
        new_overlays.push(OverlayInfo {
            mount_dir: PathBuf::from("/"),
            image_path: sdk.to_path_buf(),
        });
    }
    new_overlays.extend(overlays.iter().map(|overlay| OverlayInfo {
        image_path: overlay.image_path.to_path_buf(),
        mount_dir: if overlay.mount_dir.is_absolute() {
            overlay.mount_dir.to_path_buf()
        } else {
            PathBuf::from(SOURCE_DIR).join(overlay.mount_dir.to_path_buf())
        },
    }));
    return Ok(Config {
        overlays: new_overlays,
        login_mode,
        remounts: Vec::new(),
        bind_mounts: Vec::new(),
        run_in_container_extra_args: Vec::new(),
    });
}
