// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Result};
use fileutil::get_user_xattrs_map;
use itertools::Itertools;
use serde::Serialize;
use sha2::{Digest, Sha256};

use std::{
    collections::BTreeMap,
    fs::{read_link, File},
    os::unix::prelude::*,
    path::{Path, PathBuf},
};

const MODE_MASK: u32 = 0o7777;

/// Keeps track of permissions of a file and restores it automatically.
///
/// It records the permissions of a file on [`SavedPermissions::try_new`]. You
/// can call its methods to change the permissions. Finally, dropping the
/// instance, or calling [`SavedPermissions::restore`] explicitly, restores
/// the permissions to the original ones. It panics if it encounters an error on
/// drop.
pub struct SavedPermissions {
    path: PathBuf,
    original: u32,
    current: u32,
}

impl SavedPermissions {
    /// Creates a new instance of [`SavedPermissions`]. It records the current
    /// permissions of the file.
    pub fn try_new(path: &Path) -> Result<Self> {
        let metadata = std::fs::metadata(path)?;
        let mode = metadata.mode() & MODE_MASK;
        Ok(Self {
            path: path.to_owned(),
            original: mode,
            current: mode,
        })
    }

    /// Sets the permissions of the file.
    pub fn set(&mut self, mode: u32) -> Result<()> {
        if mode != self.current {
            std::fs::set_permissions(&self.path, PermissionsExt::from_mode(mode))?;
            self.current = mode;
        }
        Ok(())
    }

    /// Restores the permissions of the file to the original ones.
    pub fn restore(&mut self) -> Result<()> {
        self.set(self.original)
    }

    /// Ensures that the file is readable by its owner, just like `chmod u+r`.
    // #[cfg(test)] // currently called only from tests
    pub fn ensure_readable(&mut self) -> Result<()> {
        if self.current & 0o400 != 0o400 {
            self.set(self.current | 0o400)?;
        }
        Ok(())
    }

    /// Ensures that the file is fully accessible by its owner, just like
    /// `chmod u+rwx`.
    pub fn ensure_full_access(&mut self) -> Result<()> {
        if self.current & 0o700 != 0o700 {
            self.set(self.current | 0o700)?;
        }
        Ok(())
    }
}

impl Drop for SavedPermissions {
    fn drop(&mut self) {
        self.restore().expect("Failed to restore saved permissions");
    }
}

/// SHA256 hash of an empty data.
pub const EMPTY_HASH: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

/// Describes a file.
#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum FileDescription {
    File {
        path: PathBuf,
        mode: u32,
        hash: String,
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        user_xattrs: BTreeMap<String, Vec<u8>>,
    },
    Dir {
        path: PathBuf,
        mode: u32,
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        user_xattrs: BTreeMap<String, Vec<u8>>,
    },
    Symlink {
        path: PathBuf,
        mode: u32,
        target: PathBuf,
    },
    Char {
        path: PathBuf,
        mode: u32,
        rdev: u64,
    },
}

/// Helper function to create a simple [`FileDescription::Dir`].
pub fn simple_dir(path: &'static str, mode: u32) -> FileDescription {
    FileDescription::Dir {
        path: PathBuf::from(path),
        mode,
        user_xattrs: [].into(),
    }
}

/// Helper function to create a simple [`FileDescription::File`].
pub fn simple_file(path: &'static str, mode: u32, hash: &'static str) -> FileDescription {
    FileDescription::File {
        path: PathBuf::from(path),
        mode,
        hash: hash.to_owned(),
        user_xattrs: [].into(),
    }
}

fn describe_tree_impl(
    root_dir: &Path,
    relative_path: &Path,
    files: &mut Vec<FileDescription>,
) -> Result<()> {
    let full_path = root_dir.join(relative_path);
    let metadata = std::fs::symlink_metadata(&full_path)?;
    let mode = metadata.mode() & MODE_MASK;
    let file_type = metadata.file_type();

    if file_type.is_file() {
        let mut perms = SavedPermissions::try_new(&full_path)?;
        perms.ensure_readable()?;

        let mut file = File::open(&full_path)?;
        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher)?;
        let hash = hex::encode(hasher.finalize());
        let user_xattrs = get_user_xattrs_map(&full_path)?;
        files.push(FileDescription::File {
            path: relative_path.to_owned(),
            mode,
            hash,
            user_xattrs,
        });
    } else if file_type.is_dir() {
        let mut perms = SavedPermissions::try_new(&full_path)?;
        perms.ensure_full_access()?;

        let user_xattrs = get_user_xattrs_map(&full_path)?;
        files.push(FileDescription::Dir {
            path: relative_path.to_owned(),
            mode,
            user_xattrs,
        });

        let entries = std::fs::read_dir(full_path)?
            .collect::<std::io::Result<Vec<_>>>()?
            .into_iter()
            // Sort entries to make the output deterministic.
            .sorted_by(|a, b| a.file_name().cmp(&b.file_name()));
        for entry in entries {
            describe_tree_impl(root_dir, &relative_path.join(entry.file_name()), files)?;
        }
    } else if file_type.is_symlink() {
        let target = read_link(&full_path)?;
        files.push(FileDescription::Symlink {
            path: relative_path.to_owned(),
            mode,
            target,
        });
    } else if file_type.is_char_device() {
        let rdev = metadata.rdev();
        files.push(FileDescription::Char {
            path: relative_path.to_owned(),
            mode,
            rdev,
        });
    } else {
        bail!("Unsupported file type: {:?}", file_type);
    }

    Ok(())
}

/// Loads all files under a directory, including contents and metadata.
/// This function is useful to compare a directory tree.
pub fn describe_tree(root_dir: &Path) -> Result<Vec<FileDescription>> {
    let mut files: Vec<FileDescription> = Vec::new();
    describe_tree_impl(root_dir, Path::new(""), &mut files)?;
    Ok(files)
}
