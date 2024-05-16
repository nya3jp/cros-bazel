// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{anyhow, bail, Context, Result};
use fileutil::get_user_xattrs_map;
use fileutil::SafeTempDirBuilder;
use itertools::Itertools;
use std::{
    fs::{rename, set_permissions, File, Metadata, Permissions},
    os::unix::prelude::*,
    path::Path,
};
use tracing::instrument;

use crate::{
    consts::{MANIFEST_FILE_NAME, MARKER_FILE_NAME, MODE_MASK, RAW_DIR_NAME},
    manifest::{DurableTreeManifest, FileEntry},
    util::DirLock,
};

/// Renames a given directory to the `raw` subdirectory under itself.
/// For example, the original directory is at `/path/to/dir`, it is renamed to
/// `/path/to/dir/files`. Importantly, directory metadata such as permissions
/// and user xattrs are preserved.
///
/// It returns an error if the given directory path is empty.
#[instrument]
fn pivot_to_raw_subdir(root_dir: &Path) -> Result<()> {
    let parent_root_dir = root_dir
        .parent()
        .ok_or_else(|| anyhow!("Directory path must not be empty"))?;

    let temp_dir = SafeTempDirBuilder::new()
        .base_dir(parent_root_dir)
        .build()
        .with_context(|| {
            format!(
                "Failed to create a temporary directory under {}",
                parent_root_dir.display()
            )
        })?;

    rename(root_dir, temp_dir.path().join(RAW_DIR_NAME))?;
    // SafeTempDir::into_path() makes the directory permanent.
    rename(temp_dir.into_path(), root_dir)?;

    Ok(())
}

/// Processes a file. It removes a file if it is neither a regular file or a directory,
/// and returns [`FileEntry`] to be recorded in the manifest file.
fn process_file(path: &Path, metadata: &Metadata) -> Result<FileEntry> {
    if metadata.is_file() || metadata.is_dir() {
        let mode = metadata.mode() & MODE_MASK;

        set_permissions(path, Permissions::from_mode(0o755))
            .with_context(|| format!("chmod 755 {}", path.display()))?;

        let user_xattrs = get_user_xattrs_map(path)?;

        Ok(if metadata.is_file() {
            FileEntry::Regular { mode, user_xattrs }
        } else {
            FileEntry::Directory { mode, user_xattrs }
        })
    } else if metadata.is_symlink() {
        let symlink_target = path
            .read_link()
            .with_context(|| format!("readlink {}", path.display()))?;
        let symlink_target = symlink_target
            .to_str()
            .with_context(|| format!("readlink {}: not valid UTF-8", path.display()))?;
        std::fs::remove_file(path).with_context(|| format!("rm {}", path.display()))?;

        Ok(FileEntry::Symlink {
            target: symlink_target.to_string(),
        })
    } else if metadata.file_type().is_char_device() && metadata.rdev() == 0 {
        std::fs::remove_file(path).with_context(|| format!("rm {}", path.display()))?;

        Ok(FileEntry::Whiteout)
    } else {
        bail!(
            "Unsupported file type {:?}: {}",
            metadata.file_type(),
            path.display()
        );
    }
}

fn build_manifest_impl(
    raw_dir: &Path,
    relative_path: &Path,
    manifest: &mut DurableTreeManifest,
) -> Result<()> {
    let path = raw_dir.join(relative_path);
    let metadata = std::fs::symlink_metadata(&path)?;

    let file_manifest = process_file(&path, &metadata)?;
    manifest.files.insert(
        relative_path
            .to_str()
            .ok_or_else(|| anyhow!("Non-UTF8 filename: {:?}", relative_path))?
            .to_owned(),
        file_manifest,
    );

    if metadata.is_dir() {
        let entries = std::fs::read_dir(&path)?
            .collect::<std::io::Result<Vec<_>>>()?
            .into_iter()
            // Sort entries to make the output deterministic.
            .sorted_by(|a, b| a.file_name().cmp(&b.file_name()));
        for entry in entries {
            build_manifest_impl(raw_dir, &relative_path.join(entry.file_name()), manifest)?;
        }
    }

    Ok(())
}

/// Scans files under the raw directory and builds a manifest JSON and an extra
/// tarball file.
#[instrument]
fn build_manifest(root_dir: &Path) -> Result<()> {
    let raw_dir = root_dir.join(RAW_DIR_NAME);
    let mut manifest: DurableTreeManifest = Default::default();

    build_manifest_impl(&raw_dir, Path::new(""), &mut manifest)?;

    serde_json::to_writer(File::create(root_dir.join(MANIFEST_FILE_NAME))?, &manifest)?;

    Ok(())
}

/// Converts a plain directory into a durable tree in place.
pub fn convert_impl(root_dir: &Path) -> Result<()> {
    // Fail on non-fully-accessible root directories. It's more complicated than
    // what one initially expects to support it, and such directory trees don't
    // appear in real use cases.
    let metadata = std::fs::metadata(root_dir)?;
    if metadata.permissions().mode() & 0o700 != 0o700 {
        bail!("{} is not fully accessible", root_dir.display());
    }

    // Lock the root directory to give a better error message on concurrently
    // calling this function on the same directory.
    let _lock = DirLock::try_new(root_dir)?;

    // Ensure that the directory is not a durable tree.
    if root_dir.join(MARKER_FILE_NAME).try_exists()? {
        bail!("{} is already a durable tree", root_dir.display());
    }

    pivot_to_raw_subdir(root_dir)?;
    build_manifest(root_dir)?;

    // Mark as hot initially.
    set_permissions(root_dir, Permissions::from_mode(0o700))?;

    // Mark as a durable tree.
    File::create(root_dir.join(MARKER_FILE_NAME))?;

    Ok(())
}
