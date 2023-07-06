// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::path::Path;

use anyhow::Result;

pub static CHROOT_SOURCE_DIR: &str = "/mnt/host/source";

/// Returns true when running inside ChromeOS SDK chroot.
pub fn is_inside_chroot() -> Result<bool> {
    Ok(Path::new("/etc/cros_chroot_version").try_exists()?)
}
