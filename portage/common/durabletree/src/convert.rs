// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{anyhow, bail, Context, Result};
use fileutil::SafeTempDirBuilder;
use itertools::Itertools;
use std::{
    collections::HashSet,
    fs::{read_link, rename, File, Metadata},
    os::unix::prelude::*,
    path::{Path, PathBuf},
};
use tar::{EntryType, Header, HeaderMode};
use tracing::instrument;

use crate::{
    consts::{
        EXTRA_TARBALL_FILE_NAME, MANIFEST_FILE_NAME, MARKER_FILE_NAME, MODE_MASK, RAW_DIR_NAME,
        RESTORED_XATTR,
    },
    manifest::{DurableTreeManifest, FileManifest},
    util::{get_user_xattrs_map, DirLock, SavedPermissions},
};

struct ExtraTarballBuilder {
    raw_dir: PathBuf,
    tar_builder: tar::Builder<zstd::Encoder<'static, File>>,
    written_dirs: HashSet<PathBuf>,
}

impl ExtraTarballBuilder {
    pub fn new(root_dir: &Path) -> Result<Self> {
        let file = File::create(root_dir.join(EXTRA_TARBALL_FILE_NAME))?;
        let zstd_encoder = zstd::Encoder::new(file, 0)?;
        let mut tar_builder = tar::Builder::new(zstd_encoder);
        tar_builder.mode(HeaderMode::Deterministic); // for reproducibility.

        let mut builder = Self {
            raw_dir: root_dir.join(RAW_DIR_NAME),
            tar_builder,
            written_dirs: HashSet::new(),
        };

        // Always include the root directory in the tarball. Otherwise we might set
        // a wrong permissions to the root directory.
        builder.ensure_ancestors(Path::new("_"))?;

        Ok(builder)
    }

    pub fn finish(self) -> Result<()> {
        let encoder = self.tar_builder.into_inner()?;
        let file = encoder.finish()?;
        file.sync_all()?;
        Ok(())
    }

    pub fn move_into_tarball(&mut self, relative_path: &Path, metadata: &Metadata) -> Result<()> {
        let file_type = metadata.file_type();
        let dot_relative_path = Path::new(".").join(relative_path);

        self.ensure_ancestors(relative_path)?;

        if file_type.is_file() || file_type.is_dir() {
            bail!("Regular files and directories are not supported in extra tarballs");
        } else if file_type.is_symlink() {
            let target = read_link(self.raw_dir.join(relative_path))?;

            let mut header = Header::new_gnu();
            header.set_entry_type(EntryType::Symlink);
            header.set_mode(metadata.mode() & MODE_MASK);
            self.tar_builder
                .append_link(&mut header, dot_relative_path, target)?;
        } else if file_type.is_char_device() {
            if metadata.rdev() != 0 {
                bail!(
                    "Unsupported character device file (rdev=0x{:x}); \
                    only whiteout files can be created without CAP_MKNOD",
                    metadata.rdev()
                );
            }

            let mut header = Header::new_gnu();
            header.set_path(dot_relative_path)?;
            header.set_entry_type(EntryType::Char);
            header.set_device_major(0)?;
            header.set_device_minor(0)?;
            header.set_mode(metadata.mode() & MODE_MASK);
            header.set_cksum();
            self.tar_builder.append(&header, &[] as &[u8])?;
        } else {
            bail!("Unsupported file type {:?}", file_type);
        }

        fileutil::remove_file_with_chmod(&self.raw_dir.join(relative_path))?;
        Ok(())
    }

    fn ensure_ancestors(&mut self, relative_path: &Path) -> Result<()> {
        // Write all ancestor directories if not written yet.
        // rev() to write parents before children.
        for dir in relative_path
            .ancestors()
            .skip(1)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            if self.written_dirs.contains(dir) {
                break;
            }
            let metadata = self.raw_dir.join(dir).metadata()?;
            let mut header = Header::new_gnu();
            header.set_path(Path::new(".").join(dir))?;
            header.set_entry_type(EntryType::Directory);
            header.set_mode(metadata.mode() & MODE_MASK);
            header.set_cksum();
            // Don't use `append_dir` as it drops some mode bits.
            self.tar_builder.append(&header, &[] as &[u8])?;
            self.written_dirs.insert(dir.to_owned());
        }
        Ok(())
    }
}

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

fn build_manifest_and_extra_tarball_impl(
    raw_dir: &Path,
    relative_path: &Path,
    manifest: &mut DurableTreeManifest,
    extra_builder: &mut ExtraTarballBuilder,
) -> Result<()> {
    let full_path = raw_dir.join(relative_path);
    let metadata = std::fs::symlink_metadata(&full_path)?;

    if metadata.is_file() || metadata.is_dir() {
        let mode = metadata.mode() & MODE_MASK;
        let user_xattrs = get_user_xattrs_map(&full_path)?;
        manifest.files.insert(
            relative_path
                .to_str()
                .ok_or_else(|| anyhow!("Non-UTF8 filename: {:?}", relative_path))?
                .to_owned(),
            FileManifest { mode, user_xattrs },
        );
    } else {
        extra_builder.move_into_tarball(relative_path, &metadata)?;
    }

    if metadata.is_dir() {
        let mut perms = SavedPermissions::try_new(&full_path)?;
        perms.ensure_full_access()?;

        let entries = std::fs::read_dir(full_path)?
            .collect::<std::io::Result<Vec<_>>>()?
            .into_iter()
            // Sort entries to make the output deterministic.
            .sorted_by(|a, b| a.file_name().cmp(&b.file_name()));
        for entry in entries {
            build_manifest_and_extra_tarball_impl(
                raw_dir,
                &relative_path.join(entry.file_name()),
                manifest,
                extra_builder,
            )?;
        }
    }

    Ok(())
}

/// Scans files under the raw directory and builds a manifest JSON and an extra
/// tarball file.
#[instrument]
fn build_manifest_and_extra_tarball(root_dir: &Path) -> Result<()> {
    let raw_dir = root_dir.join(RAW_DIR_NAME);

    let mut manifest: DurableTreeManifest = Default::default();
    let mut extra_builder = ExtraTarballBuilder::new(root_dir)?;

    build_manifest_and_extra_tarball_impl(
        &raw_dir,
        Path::new(""),
        &mut manifest,
        &mut extra_builder,
    )?;

    serde_json::to_writer(File::create(root_dir.join(MANIFEST_FILE_NAME))?, &manifest)?;
    extra_builder.finish()?;

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
    build_manifest_and_extra_tarball(root_dir)?;

    // Mark as restored initially.
    xattr::set(root_dir, RESTORED_XATTR, &[] as &[u8])?;

    // Mark as a durable tree.
    File::create(root_dir.join(MARKER_FILE_NAME))?;

    Ok(())
}