// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    fs::{create_dir, create_dir_all, read_dir},
    io::ErrorKind,
    iter::once,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use nix::{
    mount::{mount, MsFlags},
    sched::{unshare, CloneFlags},
    unistd::{getgid, getuid, pivot_root},
};
use walkdir::WalkDir;

struct SymlinkPlan {
    pub source: PathBuf,
    pub target: PathBuf,
}

/// Enters a fake CrOS chroot.
///
/// A fake CrOS chroot is not a real CrOS chroot, but it's more like a unified
/// view of a part of the CrOS chroot and the original system environment.
/// Specifically, a fake CrOS chroot provides /mnt/host/source, /build,
/// /etc/portage and several other files in the CrOS chroot needed to evaluate
/// Portage profiles and ebuilds. However, the process can still access other
/// file paths on the system, e.g. Bazel runfiles.
///
/// This function requires the current process to be single-threaded for
/// unshare(2) calls to succeed. Make sure to call this function early in your
/// program, before starting threads.
pub fn enter_fake_chroot(source_dir: impl AsRef<Path>) -> Result<()> {
    let old_root_name = ".old-root";
    let root_dir = PathBuf::from("/");

    let source_dir = source_dir.as_ref();
    let chroot_dir = source_dir.join("chroot");

    // Enter a new namespace.
    let uid = getuid();
    let gid = getgid();
    unshare(CloneFlags::CLONE_NEWUSER | CloneFlags::CLONE_NEWNS)
        .with_context(|| format!("unshare(2) failed"))?;
    std::fs::write("/proc/self/setgroups", "deny")
        .with_context(|| "Writing /proc/self/setgroups")?;
    std::fs::write("/proc/self/uid_map", format!("0 {} 1\n", uid))
        .with_context(|| "Writing /proc/self/uid_map")?;
    std::fs::write("/proc/self/gid_map", format!("0 {} 1\n", gid))
        .with_context(|| "Writing /proc/self/gid_map")?;

    // Make a temporary directory that would be the new root.
    let new_root_dir = tempfile::tempdir_in("/tmp")?;
    let new_root_dir = new_root_dir.path();

    // Mount tmpfs on the temporary directory so that symlinks we are creating
    // from now will be removed at the end of the namespace.
    mount(
        Some(""),
        new_root_dir,
        Some("tmpfs"),
        MsFlags::empty(),
        Some(""),
    )
    .with_context(|| format!("mount(2) failed on mounting tmpfs"))?;

    // Create symlinks to overriding files in chroot.
    let simple_symlink_paths = &[
        "build",
        "etc/make.conf",
        "etc/make.conf.board_setup",
        "etc/make.conf.host_setup",
        "etc/make.conf.user",
        "etc/portage",
    ];

    let plans = once(SymlinkPlan {
        source: new_root_dir.join("mnt/host/source"),
        target: source_dir.to_owned(),
    })
    .chain(simple_symlink_paths.iter().map(|path| SymlinkPlan {
        source: new_root_dir.join(path),
        target: chroot_dir.join(path),
    }));

    for plan in plans {
        create_dir_all(plan.source.parent().unwrap()).with_context(|| {
            format!(
                "Creating directories: {}",
                plan.source.parent().unwrap().to_string_lossy()
            )
        })?;
        symlink(&plan.target, &plan.source).with_context(|| {
            format!(
                "Creating symlink: {} -> {}",
                plan.target.to_string_lossy(),
                plan.source.to_string_lossy()
            )
        })?;
    }

    // Create symlinks to files in the original filesystem.
    // The old root filesystem will be mounted at /.old-root. Since we have
    // created several symlinks to override the original filesystem, we will
    // create symlinks to files that exist in the the original filesystem but
    // not in [new_root_dir].
    for new_dir_entry in WalkDir::new(new_root_dir) {
        // Iterate on all directories under [new_root_dir].
        // Example: ${new_root_dir}/etc
        let new_dir_entry = new_dir_entry?;
        if !new_dir_entry.file_type().is_dir() {
            continue;
        }

        let rel_path = new_dir_entry.path().strip_prefix(new_root_dir)?;
        let orig_dir = root_dir.join(rel_path);

        // Enumerate files in the corresponding directory in the original
        // filesystem.
        // Example: /etc
        let orig_dir_entries = {
            match read_dir(&orig_dir) {
                Ok(entries) => entries,
                Err(e) if e.kind() == ErrorKind::NotFound => {
                    // If the directory does not exist in the original
                    // filesystem, we can skip this directory.
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        };

        // Create symlinks to /.old-root for enumerated files, except when
        // conflicting symlinks were already created in previous steps.
        // Example: ${new_root_dir}/etc/resolv.conf -> /.old-root/etc/resolv.conf
        for orig_dir_entry in orig_dir_entries {
            let orig_dir_entry = orig_dir_entry?;
            let source = new_dir_entry.path().join(orig_dir_entry.file_name());
            let target = root_dir
                .join(old_root_name)
                .join(rel_path)
                .join(orig_dir_entry.file_name());
            match symlink(target, source) {
                Ok(_) => {}
                Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
                Err(e) => {
                    return Err(e.into());
                }
            };
        }
    }

    // Create the directory to mount the old filesystem root after
    // pivot_root(2).
    create_dir(new_root_dir.join(old_root_name))?;

    // Finally, call pivot_root(2).
    pivot_root(new_root_dir, &new_root_dir.join(old_root_name))?;

    Ok(())
}
