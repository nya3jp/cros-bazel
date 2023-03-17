// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use libc::S_IWUSR;
use std::fs::Permissions;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

/// Moves the contents of `from` to `to` after ensuring we have `u+w` to each directory entry and
/// restores the original file permissions.
pub fn move_dir_contents(from: &Path, to: &Path) -> Result<()> {
    for entry in std::fs::read_dir(from).with_context(|| format!("Failed to read dir {from:?}"))? {
        let entry = entry?;
        let src = from.to_path_buf().join(entry.file_name());
        let dest = to.to_path_buf().join(entry.file_name());

        let metadata = entry.metadata()?;
        let new_mode = metadata.mode() | S_IWUSR;

        // For directories, we need u+w (S_IWUSR) permission to rename.
        if metadata.is_dir() && metadata.mode() != new_mode {
            std::fs::set_permissions(&src, Permissions::from_mode(new_mode)).with_context(
                || format!("Failed to set permission for {:?} to {:o}", &src, new_mode),
            )?;
        }

        std::fs::rename(&src, &dest)
            .with_context(|| format!("Failed to rename {:?} to {:?}", &src, &dest))?;

        if metadata.is_dir() && metadata.mode() != new_mode {
            std::fs::set_permissions(&dest, Permissions::from_mode(metadata.mode())).with_context(
                || {
                    format!(
                        "Failed to restore permission of {:?} to {:o}",
                        &dest,
                        metadata.mode()
                    )
                },
            )?;
        }
    }
    Ok(())
}
