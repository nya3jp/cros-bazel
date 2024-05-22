// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::BTreeMap,
    fs::{remove_dir, set_permissions, File},
    io::{BufReader, ErrorKind},
    os::unix::{fs::symlink, prelude::PermissionsExt},
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use fileutil::{with_permissions, SafeTempDir};
use nix::{
    mount::{mount, umount2, MntFlags, MsFlags},
    sys::stat::{mknod, Mode, SFlag},
};
use tracing::{instrument, warn};

use crate::{
    consts::{MANIFEST_FILE_NAME, MARKER_FILE_NAME, RAW_DIR_NAME, RESTORED_XATTR},
    manifest::{DurableTreeManifest, FileEntry},
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

fn restore_user_xattrs_and_permissions(
    path: &Path,
    mode: u32,
    user_xattrs: &BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    // First set a writable permission for restoring xattrs.
    set_permissions(path, PermissionsExt::from_mode(0o755))
        .with_context(|| format!("chmod 755 {}", path.display()))?;

    // Restore user xattrs.
    for (key, value) in user_xattrs {
        xattr::set(path, key, value).with_context(|| format!("setxattr {}", path.display()))?;
    }

    // Restore permissions.
    set_permissions(path, PermissionsExt::from_mode(mode))
        .with_context(|| format!("chmod {:03o} {}", mode, path.display()))?;

    Ok(())
}

/// Restores the raw directory if not yet.
#[instrument(skip(manifest))] // don't record the huge manifest argument
fn maybe_restore_raw_directory(root_dir: &Path, manifest: &DurableTreeManifest) -> Result<()> {
    // Check if the raw directory is already restored.
    if let Ok(Some(_)) = xattr::get(root_dir, RESTORED_XATTR) {
        // Already restored.
        return Ok(());
    }

    let raw_dir = root_dir.join(RAW_DIR_NAME);

    for (relative_path, entry) in &manifest.files {
        let path = raw_dir.join(relative_path);

        match entry {
            FileEntry::Regular { mode, user_xattrs } => {
                restore_user_xattrs_and_permissions(&path, *mode, user_xattrs)?;
            }
            FileEntry::Directory { mode, user_xattrs } => {
                // Recreate directories as Bazel might delete empty ones.
                match std::fs::create_dir(&path) {
                    Err(err) if err.kind() == ErrorKind::AlreadyExists => {}
                    other => other.with_context(|| format!("mkdir {}", path.display()))?,
                };
                restore_user_xattrs_and_permissions(&path, *mode, user_xattrs)?;
            }
            FileEntry::Symlink { .. } | FileEntry::Whiteout => {}
        }
    }

    // Mark as restored.
    with_permissions(root_dir, 0o755, || {
        xattr::set(root_dir, RESTORED_XATTR, &[] as &[u8])
            .with_context(|| format!("Failed to set {} on {}", RESTORED_XATTR, root_dir.display()))
    })?;

    Ok(())
}

/// Create an extra directory containing special files (symlinks and whiteouts).
#[instrument(skip(manifest))] // don't record the huge manifest argument
fn create_extra_dir(manifest: &DurableTreeManifest) -> Result<ExtraDir> {
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

    for (relative_path, entry) in &manifest.files {
        let path = extra_dir.path().join(relative_path);
        let parent_dir = path.parent().unwrap();

        match entry {
            FileEntry::Regular { .. } | FileEntry::Directory { .. } => {}
            FileEntry::Symlink { target } => {
                std::fs::create_dir_all(parent_dir)
                    .with_context(|| format!("mkdir -p {}", parent_dir.display()))?;
                symlink(target, &path)
                    .with_context(|| format!("ln -s {} {}", target, path.display()))?;
            }
            FileEntry::Whiteout => {
                std::fs::create_dir_all(parent_dir)
                    .with_context(|| format!("mkdir -p {}", parent_dir.display()))?;
                mknod(&path, SFlag::S_IFCHR, Mode::from_bits(0o644).unwrap(), 0)
                    .with_context(|| format!("mknod {} c 0 0", path.display()))?;
            }
        }
    }

    Ok(extra_dir)
}

pub fn expand_impl(root_dir: &Path) -> Result<ExtraDir> {
    // Ensure that the directory is a durable tree.
    if !root_dir.join(MARKER_FILE_NAME).try_exists()? {
        bail!("{} is not a durable tree", root_dir.display());
    }

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

    let manifest: DurableTreeManifest = serde_json::from_reader(BufReader::new(File::open(
        root_dir.join(MANIFEST_FILE_NAME),
    )?))?;

    maybe_restore_raw_directory(root_dir, &manifest)?;

    create_extra_dir(&manifest)
}
