// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Joins an absolute `path` to `root`.
pub fn join_absolute(root: &Path, path: &Path) -> Result<PathBuf> {
    Ok(root.join(
        path.strip_prefix("/")
            .with_context(|| format!("path {} is not absolute", path.display()))?,
    ))
}
