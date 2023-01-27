// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::common::CHROOT_SOURCE_DIR;
use crate::fileops::execute_file_ops;
use crate::fileops::FileOps;
use std::{
    fs::{create_dir, read_dir},
    io::ErrorKind,
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

#[derive(Clone, Debug)]
pub struct PathTranslator {
    inner: PathBuf,
    outer: PathBuf,
}
/// Provides a way to translate paths inner and outer paths.
/// This is useful when running inside a container and you have a "bind mount"
/// between the host and container.
impl PathTranslator {
    fn new(inner: impl AsRef<Path>, outer: impl AsRef<Path>) -> Self {
        Self {
            inner: inner.as_ref().to_owned(),
            outer: outer.as_ref().to_owned(),
        }
    }

    pub fn to_outer(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
        let path = path.as_ref();
        if path.starts_with(&self.outer) {
            return Ok(path.to_path_buf());
        }

        let remaining = path.strip_prefix(&self.inner).with_context(|| {
            format!(
                "Cannot convert non-inner path {} to outer path. Must have a {} prefix.",
                path.display(),
                self.inner.display()
            )
        })?;

        Ok(self.outer.join(remaining))
    }

    pub fn to_inner(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
        let path = path.as_ref();
        if path.starts_with(&self.inner) {
            return Ok(path.to_path_buf());
        }

        let remaining = path.strip_prefix(&self.outer).with_context(|| {
            format!(
                "Cannot convert non-outer path {} to inner path. Must have a {} prefix.",
                path.display(),
                self.outer.display()
            )
        })?;

        Ok(self.inner.join(remaining))
    }
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
///
/// # Arguments
///
/// * `source_dir` - The `repo` root directory. i.e., directory that contains
///   the `.repo` directory. This will be mounted at /mnt/host/source.
/// * `generate_config` - A function that generates the files for the new root.
///
/// It returns [`PathTranslator`] that can be used to translate file paths in
/// the fake chroot to the original paths.
pub fn enter_fake_chroot(
    source_dir: impl AsRef<Path>,
    generate_config: &dyn Fn(&Path, &PathTranslator) -> Result<()>,
) -> Result<PathTranslator> {
    let old_root_name = ".old-root";
    let root_dir = PathBuf::from("/");

    // Canonicalize `source_dir` so it can be used in symlinks.
    let source_dir = source_dir.as_ref().canonicalize()?;

    let translator = PathTranslator::new(CHROOT_SOURCE_DIR, &source_dir);

    // Enter a new namespace.
    let uid = getuid();
    let gid = getgid();
    unshare(CloneFlags::CLONE_NEWUSER | CloneFlags::CLONE_NEWNS)
        .with_context(|| "unshare(2) failed")?;
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
    .with_context(|| "mount(2) failed on mounting tmpfs")?;

    execute_file_ops(
        &[FileOps::symlink(
            CHROOT_SOURCE_DIR,
            root_dir
                .join(old_root_name)
                .join(source_dir.strip_prefix("/")?),
        )],
        new_root_dir,
    )?;

    generate_config(new_root_dir, &translator)?;

    // We want to ensure we don't symlink the contents of the following
    // directories into the new root since we don't want any of the hosts
    // files interfering.
    let skip_mounting = [Path::new("/build"), Path::new("/mnt/host")];

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

        if skip_mounting.iter().any(|p| *p == orig_dir) {
            continue;
        }

        // Enumerate files in the corresponding directory in the original
        // filesystem.
        // Example: /etc
        let orig_dir_entries = {
            match read_dir(orig_dir) {
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

    Ok(translator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_translator() -> Result<()> {
        let translator = PathTranslator::new(CHROOT_SOURCE_DIR, "/home/cros");

        assert_eq!(
            translator.to_outer(Path::new(CHROOT_SOURCE_DIR).join("src/BUILD.bazel"))?,
            Path::new("/home/cros/src/BUILD.bazel"),
        );

        assert_eq!(
            translator.to_inner("/home/cros/src/BUILD.bazel")?,
            Path::new(CHROOT_SOURCE_DIR).join("src/BUILD.bazel")
        );

        assert!(translator.to_outer(Path::new("/etc/make.conf")).is_err());
        assert!(translator.to_inner(Path::new("/etc/make.conf")).is_err());

        Ok(())
    }
}
