// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::BTreeMap,
    ffi::OsString,
    os::{fd::AsRawFd, unix::prelude::*},
    path::{Path, PathBuf},
};

use anyhow::Result;
use nix::{
    dir::Dir,
    fcntl::{flock, FlockArg, OFlag},
    sys::stat::Mode,
};

use crate::consts::MODE_MASK;

/// Represents a lock on a directory.
///
/// The lock is released on drop.
pub struct DirLock {
    _fd: Dir,
}

impl DirLock {
    /// Acquires a new exclusive lock on a directory.
    pub fn try_new(dir: &Path) -> Result<DirLock> {
        let fd = Dir::open(dir, OFlag::O_DIRECTORY | OFlag::O_CLOEXEC, Mode::empty())?;
        flock(fd.as_raw_fd(), FlockArg::LockExclusive)?;
        Ok(DirLock { _fd: fd })
    }
}

/// Enumerates all user xattrs of a file.
pub fn list_user_xattrs(path: &Path) -> Result<Vec<OsString>> {
    let mut keys: Vec<OsString> = Vec::new();
    for key in xattr::list(path)? {
        if key.to_string_lossy().starts_with("user.") {
            keys.push(key);
        }
    }
    Ok(keys)
}

/// Returns all user xattrs of a file as a [`BTreeMap`].
pub fn get_user_xattrs_map(path: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut xattrs: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for raw_key in list_user_xattrs(path)? {
        let value = xattr::get(path, &raw_key)?.unwrap_or_default();
        let key = String::from_utf8(raw_key.as_bytes().to_owned())?;
        xattrs.insert(key, value);
    }
    Ok(xattrs)
}

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
    #[cfg(test)] // currently called only from tests
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
