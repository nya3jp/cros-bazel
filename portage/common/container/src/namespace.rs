// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{ensure, Context, Result};
use nix::{
    errno::Errno,
    mount::{mount, MsFlags},
    sched::{unshare, CloneFlags},
    unistd::{getgid, getuid},
};

fn ensure_single_threaded() -> Result<()> {
    let entries: Vec<_> = std::fs::read_dir("/proc/self/task")?.collect::<std::io::Result<_>>()?;
    ensure!(entries.len() == 1, "The current process is multi-threaded");
    Ok(())
}

fn enter_unprivileged_user_namespace() -> Result<()> {
    let uid = getuid();
    let gid = getgid();
    unshare(CloneFlags::CLONE_NEWUSER)
        .with_context(|| "Failed to create an unprivileged user namespace")?;
    std::fs::write("/proc/self/setgroups", "deny")
        .with_context(|| "Writing /proc/self/setgroups")?;
    std::fs::write("/proc/self/uid_map", format!("0 {uid} 1\n"))
        .with_context(|| "Writing /proc/self/uid_map")?;
    std::fs::write("/proc/self/gid_map", format!("0 {gid} 1\n"))
        .with_context(|| "Writing /proc/self/gid_map")?;
    Ok(())
}

/// Enters a mount namespace so that the current process can mount some file
/// systems such as tmpfs.
///
/// If the current process is unprivileged, it also enters an unprivileged user
/// namespace. Since multi-threaded processes cannot enter a new user namespace,
/// it is always an error to call this function after spawning a thread,
/// regardless of whether the current process has privilege to directly enter a
/// mount namespace.
pub fn enter_mount_namespace() -> Result<()> {
    ensure_single_threaded()?;

    match unshare(CloneFlags::CLONE_NEWNS) {
        Err(Errno::EPERM) => {
            // If the current process does not have privilege, enter an
            // unprivileged user namespace and try it again.
            enter_unprivileged_user_namespace()?;
            unshare(CloneFlags::CLONE_NEWNS)
        }
        other => other,
    }
    .context("Failed to enter a mount namespace")?;

    // Remount all file systems as private so that we never interact with the
    // original namespace. This is needed when the current process is privileged
    // and did not enter an unprivileged user namespace.
    mount(
        Some(""),
        "/",
        Some(""),
        MsFlags::MS_PRIVATE | MsFlags::MS_REC,
        Some(""),
    )
    .context("Failed to remount file systems as private")?;

    Ok(())
}
