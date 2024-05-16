// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{os::fd::AsRawFd, path::Path};

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
