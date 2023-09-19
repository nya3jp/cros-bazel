// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Result};
use nix::{
    dir::Dir,
    fcntl::{flock, FlockArg, OFlag},
    sys::stat::Mode,
};
use std::{
    fs::Permissions,
    os::{fd::AsRawFd, unix::prelude::PermissionsExt},
    path::Path,
    process::Command,
};

/// Represents a lock on a directory.
///
/// The lock is released on drop.
///
/// NOTE: This is a copy-paste from durabletree/src/util.rs for temporary use.
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

fn check_durable_tree(root_dir: &Path) -> Result<()> {
    // Resolve symlinks.
    let root_dir = root_dir.canonicalize()?;
    let root_dir = root_dir.as_path();

    // Return early if this is not a durable tree.
    if !root_dir.join("DURABLE_TREE").exists() {
        return Ok(());
    }

    // Lock the root directory to avoid restoring the directory in parallel.
    let _lock = DirLock::try_new(root_dir)?;

    // Check if the durable tree is hot.
    let metadata = std::fs::metadata(root_dir)?;
    let mode = metadata.permissions().mode() & 0o777;
    if mode == 0o555 {
        return Ok(());
    }

    println!(
        "action_wrapper: b/299934607: durable tree is hot: {}",
        root_dir.display()
    );
    println!("action_wrapper: installing trapfs");

    // Allow write access to the mount point so that fuse successfully mounts.
    let mount_dir = root_dir.join("trapfs_mount_point_b299934607");
    std::fs::set_permissions(&mount_dir, Permissions::from_mode(0o755))?;

    // Mount trapfs and wait for setattr calls.
    let trapfs_path =
        runfiles::Runfiles::create()?.rlocation("cros/bazel/portage/bin/trapfs/trapfs_/trapfs");
    let status = Command::new(trapfs_path).arg(&mount_dir).status()?;
    bail!("trapfs exited: {:?}", status);
}

pub fn check_durable_trees(args: &[String]) -> Result<()> {
    for arg in args {
        if let Some(path) = arg.strip_prefix("--layer=") {
            check_durable_tree(Path::new(path))?;
        }
    }
    Ok(())
}
