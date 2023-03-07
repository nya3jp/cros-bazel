// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use std::fs::{metadata, remove_dir_all, remove_file, set_permissions, Permissions};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

const S_IRWXU: u32 = 0o700;

/// Runs |action| after adding |permissions| to |path|. After that, restores the original
/// permissions.
fn with_permissions(
    path: &Path,
    permissions: u32,
    action: impl FnOnce() -> Result<()>,
) -> Result<()> {
    let mode = metadata(path)?.mode();
    let new_mode = mode | permissions;

    if mode != new_mode {
        set_permissions(path, Permissions::from_mode(new_mode)).with_context(|| {
            format!("Failed to set permissions for {:?} to {:o}", path, new_mode)
        })?;
    }

    let result = action();

    if mode != new_mode {
        set_permissions(path, Permissions::from_mode(mode)).with_context(|| {
            format!("Failed to restore permissions of {:?} to {:o}", path, mode)
        })?;
    }

    result
}

/// Calls `remove_file` after ensuring we have `u+rwx` to the parent directory and restores the
/// original file permissions.
pub fn remove_file_with_chmod(path: &Path) -> Result<()> {
    let parent = path.parent().unwrap();
    with_permissions(parent, S_IRWXU, || {
        remove_file(path).with_context(|| format!("Failed to delete {:?}", path))
    })
}

/// Calls `remove_dir_all` after ensuring we have `u+rwx` to each directory so that we can remove
/// all files.
pub fn remove_dir_all_with_chmod(path: &Path) -> Result<()> {
    // Make sure the directory exists before trying to walk it.
    if let Err(e) = metadata(path) {
        if e.kind() == std::io::ErrorKind::NotFound {
            return Ok(());
        }
        return Err(anyhow::Error::new(e));
    }

    for entry in WalkDir::new(&path)
        .into_iter()
        // walk isn't lazy, so if we have a directory with no permissions, it attempts to list its
        // contents (which fails since it has no permissions), then sets permissions.
        // Thus, we filter out any failures in the listing directory stage.
        // If it's an issue, it'll end up being picked up by remove_dir_all anyway.
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let mode = entry.metadata()?.mode();
        if mode & S_IRWXU != S_IRWXU {
            let new_mode = mode | S_IRWXU;
            set_permissions(entry.path(), Permissions::from_mode(new_mode)).with_context(|| {
                format!("Failed to set permissions for {:?} to {:o}", path, new_mode)
            })?;
        }
    }

    // Ensure we have u+rwx on the parent directory so we can unlink any files
    let parent = path.parent().unwrap();
    with_permissions(parent, S_IRWXU, || {
        remove_dir_all(&path).with_context(|| format!("Failed to delete {:?}", path))
    })
}

/// Removes the directory with the root privilege using sudo.
pub fn remove_dir_all_with_sudo(path: &Path) -> Result<()> {
    let status = Command::new("/usr/bin/sudo")
        .args(["rm", "-rf", "--"])
        .arg(path)
        .status()
        .with_context(|| format!("sudo rm -rf {}", path.display()))?;
    if !status.success() {
        bail!("{:?}: sudo rm -rf {}", status, path.display());
    }
    Ok(())
}
