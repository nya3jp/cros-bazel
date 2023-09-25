// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    fs::{create_dir, remove_dir, set_permissions, symlink_metadata, File},
    io::ErrorKind,
    os::unix::prelude::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

use anyhow::{bail, Context, Result};
use fileutil::SafeTempDir;
use nix::mount::{mount, umount2, MntFlags, MsFlags};
use tracing::{instrument, warn};

use crate::{
    b299934607,
    consts::{
        EXTRA_TARBALL_FILE_NAME, MANIFEST_FILE_NAME, MARKER_FILE_NAME, RAW_DIR_NAME, RESTORED_XATTR,
    },
    manifest::DurableTreeManifest,
    util::DirLock,
};

pub struct ExtraDir {
    dir: Option<PathBuf>,
}

impl ExtraDir {
    pub fn path(&self) -> &Path {
        self.dir.as_ref().unwrap()
    }
}

impl Drop for ExtraDir {
    fn drop(&mut self) {
        if let Some(dir) = &self.dir {
            umount2(dir, MntFlags::MNT_DETACH).expect("Failed to unmount tmpfs");
            remove_dir(dir).expect("Failed to remove empty dir");
        }
    }
}

/// Restores the raw directory if not yet.
#[instrument]
fn maybe_restore_raw_directory(root_dir: &Path) -> Result<()> {
    // Resolve symlinks. Otherwise xattr syscalls might attempt to read/update
    // xattrs of symlinks, not their destination.
    let root_dir = root_dir.canonicalize()?;
    let root_dir = root_dir.as_path();

    // Lock the root directory to avoid restoring the directory in parallel.
    let _lock = DirLock::try_new(root_dir)?;

    // Ensure the durable tree is not hot. See the comment in `DurableTree` for
    // details.
    let metadata = std::fs::metadata(root_dir)?;
    let mode = metadata.permissions().mode() & 0o777;
    if mode == 0o700 {
        bail!(
            "Durable tree is hot (mode = 0{:03o} != 0555): {}\n\
            Did you attempt to expand the durable tree within the same Bazel action?",
            mode,
            root_dir.display(),
        );
    }

    // Check if the raw directory is already restored.
    if let Ok(Some(_)) = xattr::get(root_dir, RESTORED_XATTR) {
        // Already restored.
        return Ok(());
    }

    // Wait until Bazel finishes calling chmod on the durable tree.
    // TODO(b/299934607): Remove this hack once a fix lands to Bazel.
    b299934607::wait_chmods(root_dir, Duration::from_secs(60))?;

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
        dir: Some(dir.into_path()),
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
