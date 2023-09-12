// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::BTreeMap,
    ffi::OsString,
    os::{fd::AsRawFd, unix::prelude::*},
    path::Path,
};

use anyhow::Result;
use nix::{
    dir::Dir,
    fcntl::{flock, FlockArg, OFlag},
    sys::stat::Mode,
};
use tracing::instrument;

/// Represents a lock on a directory.
///
/// The lock is released on drop.
pub struct DirLock {
    _fd: Dir,
}

impl DirLock {
    /// Acquires a new exclusive lock on a directory.
    #[instrument]
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
