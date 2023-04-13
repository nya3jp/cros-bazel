// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Result};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{read_link, set_permissions, File},
    os::unix::prelude::*,
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;

use crate::{
    consts::MODE_MASK,
    util::{get_user_xattrs_map, list_user_xattrs},
};

/// A helper trait to implement `Command::run_ok`.
pub trait CommandRunOk {
    /// Runs a command and ensures it exits with success.
    fn run_ok(&mut self) -> Result<()>;
}

impl CommandRunOk for Command {
    fn run_ok(&mut self) -> Result<()> {
        let status = self.status()?;
        if !status.success() {
            bail!("Command exited with {:?}", status);
        }
        Ok(())
    }
}

/// Resets metadata of files under a directory as if Bazel does.
/// Call this function between [`DurableTree::convert`] and
/// [`DurableTree::expand`] in your tests to verify that the durable tree is
/// reproducible without metadata.
pub fn reset_metadata(root_dir: &Path) -> Result<()> {
    for entry in WalkDir::new(root_dir) {
        let entry = entry?;
        let path = entry.path();

        // Make the file writable allow listing/clearing user xattrs.
        set_permissions(path, PermissionsExt::from_mode(0o755))?;

        // Clear all user xattrs.
        for key in list_user_xattrs(path)? {
            xattr::remove(path, key)?;
        }

        // Set the file permission to 0555 that Bazel typically uses.
        set_permissions(path, PermissionsExt::from_mode(0o555))?;
    }
    Ok(())
}

/// SHA256 hash of an empty data.
pub const EMPTY_HASH: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

/// Describes a file.
#[derive(Debug, Eq, PartialEq)]
pub enum FileDescription {
    File {
        path: PathBuf,
        mode: u32,
        hash: String,
        user_xattrs: BTreeMap<String, Vec<u8>>,
    },
    Dir {
        path: PathBuf,
        mode: u32,
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

/// Loads all files under a directory, including contents and metadata.
/// This function is useful to compare a directory tree.
pub fn describe_tree(root_dir: &Path) -> Result<Vec<FileDescription>> {
    let mut files: Vec<FileDescription> = Vec::new();

    for entry in WalkDir::new(root_dir).sort_by_file_name() {
        let entry = entry?;
        let path = entry.path().strip_prefix(root_dir)?.to_owned();
        let metadata = entry.metadata()?;
        let mode = metadata.mode() & MODE_MASK;
        let file_type = metadata.file_type();

        if file_type.is_file() {
            let mut file = File::open(entry.path())?;
            let mut hasher = Sha256::new();
            std::io::copy(&mut file, &mut hasher)?;
            let hash = hex::encode(hasher.finalize());
            let user_xattrs = get_user_xattrs_map(entry.path())?;
            files.push(FileDescription::File {
                path,
                mode,
                hash,
                user_xattrs,
            });
        } else if file_type.is_dir() {
            let user_xattrs = get_user_xattrs_map(entry.path())?;
            files.push(FileDescription::Dir {
                path,
                mode,
                user_xattrs,
            });
        } else if file_type.is_symlink() {
            let target = read_link(entry.path())?;
            files.push(FileDescription::Symlink { path, mode, target });
        } else if file_type.is_char_device() {
            let rdev = metadata.rdev();
            files.push(FileDescription::Char { path, mode, rdev });
        } else {
            bail!("Unsupported file type: {:?}", file_type);
        }
    }

    Ok(files)
}
