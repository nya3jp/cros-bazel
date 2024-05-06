// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use std::path::{Path, PathBuf};

pub const BINPKG: &str = "nano.tbz2";
pub const BINPKG_DIFF_XPAK: &str = "nano-diff-xpak.tbz2";
pub const BINPKG_DIFF_TAR: &str = "nano-diff-tar.tbz2";
pub const BINPKG_CLEAN_ENV: &str = "nano-clean-env.tbz2";

pub fn testdata(path: impl AsRef<Path>) -> Result<PathBuf> {
    match runfiles::Runfiles::create() {
        Ok(r) => Ok(runfiles::rlocation!(
            r,
            Path::new("cros/bazel/portage/common/testdata").join(path)
        )),
        Err(_) => Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../common/testdata")
            .join(path)),
    }
}
