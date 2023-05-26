// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    fs::{create_dir, set_permissions, symlink_metadata, File},
    io::ErrorKind,
    os::unix::prelude::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{bail, Context, Result};
use fileutil::SafeTempDir;
use nix::mount::{mount, umount2, MntFlags, MsFlags};
use tracing::instrument;

use crate::{
    consts::{
        EXTRA_TARBALL_FILE_NAME, MANIFEST_FILE_NAME, MARKER_FILE_NAME, RAW_DIR_NAME, RESTORED_XATTR,
    },
    manifest::DurableTreeManifest,
    util::DirLock,
};

pub struct ExtraDir {
    dir: PathBuf,
}

impl ExtraDir {
    pub fn path(&self) -> &Path {
        &self.dir
    }
}

impl Drop for ExtraDir {
    fn drop(&mut self) {
        umount2(self.path(), MntFlags::MNT_DETACH).expect("Failed to unmount tmpfs");
    }
}

/// Restores the raw directory if not yet.
#[instrument]
fn maybe_restore_raw_directory(root_dir: &Path) -> Result<()> {
    // Lock the root directory to avoid restoring the directory in parallel.
    let _lock = DirLock::try_new(root_dir)?;

    // Check if the raw directory is already restored.
    if let Ok(Some(_)) = xattr::get(root_dir, RESTORED_XATTR) {
        // Already restored.
        return Ok(());
    }

    let raw_dir = root_dir.join(RAW_DIR_NAME);

    let manifest: DurableTreeManifest =
        serde_json::from_reader(File::open(root_dir.join(MANIFEST_FILE_NAME))?)?;
    for (relative_path, file_manifest) in manifest.files {
        let path = raw_dir.join(&relative_path);

        // Check if the file path exists. If not, it is a directory
        // ignored by the Bazel cache.
        match symlink_metadata(&path) {
            Err(err) if err.kind() == ErrorKind::NotFound => {
                create_dir(&path)
                    .with_context(|| format!("Restoring directory {}", path.display()))?;
            }
            other => {
                other?;
            }
        };

        // Restore permissions.
        set_permissions(&path, PermissionsExt::from_mode(file_manifest.mode))
            .with_context(|| format!("Setting permissions to {}", &relative_path))?;

        // Restore user xattrs.
        for (key, value) in file_manifest.user_xattrs {
            xattr::set(&path, key, &value)
                .with_context(|| format!("Setting xattrs to {}", &relative_path))?;
        }
    }

    // Mark as restored.
    set_permissions(root_dir, PermissionsExt::from_mode(0o755))?;
    xattr::set(root_dir, RESTORED_XATTR, &[] as &[u8])?;

    Ok(())
}

/// Extracts the extra tarball into a temporary directory and returns its path.
#[instrument]
fn extract_extra_files(root_dir: &Path) -> Result<ExtraDir> {
    let dir = SafeTempDir::new()?;

    mount(
        Some(""),
        dir.path(),
        Some("tmpfs"),
        MsFlags::empty(),
        Some("mode=0755"),
    )
    .context("Failed to mount tmpfs for extra dir")?;

    let extra_dir = ExtraDir {
        dir: dir.into_path(),
    };

    // TODO: Avoid depending on the system-installed tar(1).
    // It's not too bad though as it is so popular in Linux systems.
    let mut child = Command::new("tar")
        .args(["--extract", "--preserve-permissions"])
        .current_dir(extra_dir.path())
        .stdin(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.take().unwrap();
    zstd::stream::copy_decode(File::open(root_dir.join(EXTRA_TARBALL_FILE_NAME))?, stdin)?;

    let status = child.wait()?;
    if !status.success() {
        bail!("tar failed: {:?}", status);
    }

    Ok(extra_dir)
}

pub fn expand_impl(root_dir: &Path) -> Result<ExtraDir> {
    // Ensure that the directory is a durable tree.
    if !root_dir.join(MARKER_FILE_NAME).try_exists()? {
        bail!("{} is not a durable tree", root_dir.display());
    }

    maybe_restore_raw_directory(root_dir)?;

    extract_extra_files(root_dir)
}
