// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, ensure, Context, Result};
use itertools::Itertools;
use nix::{
    mount::{mount, umount2, MntFlags, MsFlags},
    sys::statvfs::{statvfs, FsFlags},
};
use tracing::info_span;

fn ensure_dir_is_empty(dir: &Path) -> Result<()> {
    match std::fs::read_dir(dir)?.next() {
        None => Ok(()),
        Some(Ok(entry)) => bail!(
            "{} is not empty: {} exists",
            dir.display(),
            entry.file_name().to_string_lossy()
        ),
        Some(Err(e)) => Err(e.into()),
    }
}

/// Unmounts an mount point on drop.
#[must_use]
pub(crate) struct MountGuard {
    dir: Option<PathBuf>,
}

impl MountGuard {
    fn new(dir: &Path) -> Self {
        Self {
            dir: Some(dir.to_path_buf()),
        }
    }

    // Forgets this mount point. After calling this method, it is your responsibility to unmount it.
    // It is often safe to use this method, e.g. when this mount point is under another mount point
    // and you're sure it's unmounted recursively.
    pub(crate) fn leak(mut self) {
        self.dir = None;
    }
}

impl Drop for MountGuard {
    fn drop(&mut self) {
        if let Some(dir) = self.dir.take() {
            let _span = info_span!("MountGuard::drop", dir = ?dir).entered();
            umount2(&dir, MntFlags::MNT_DETACH).expect("Failed to unmount");
        }
    }
}

/// Bind-mounts given paths.
pub(crate) fn bind_mount(old_dir: &Path, new_dir: &Path) -> Result<MountGuard> {
    mount(
        Some(old_dir),
        new_dir,
        Some(""),
        MsFlags::MS_BIND | MsFlags::MS_REC,
        Some(""),
    )
    .with_context(|| {
        format!(
            "Bind-mounting {} to {} failed",
            old_dir.display(),
            new_dir.display()
        )
    })?;
    Ok(MountGuard::new(new_dir))
}

pub(crate) fn remount_readonly(path: &Path) -> Result<()> {
    let mut flags = MsFlags::MS_REMOUNT | MsFlags::MS_BIND | MsFlags::MS_RDONLY;

    let stat = statvfs(path).with_context(|| format!("Failed to statvfs: {}", path.display()))?;
    if stat.flags().contains(FsFlags::ST_NODEV) {
        flags |= MsFlags::MS_NODEV;
    }
    if stat.flags().contains(FsFlags::ST_NOEXEC) {
        flags |= MsFlags::MS_NOEXEC;
    }
    if stat.flags().contains(FsFlags::ST_NOSUID) {
        flags |= MsFlags::MS_NOSUID;
    }
    mount(Some(""), path, Some(""), flags, Some(""))
        .with_context(|| format!("Failed remounting {} as read-only", path.display()))?;
    Ok(())
}

/// Mounts overlayfs at the specified path.
///
/// `scratch_dir` should point to an empty directory where the function will
/// create arbitrary files/directories needed to mount overlayfs. The directory
/// must be on the same file system as the upper directory as an overlayfs work
/// directory is allocated there. Callers must remove the directory *after*
/// unmounting the overlayfs by dropping the returned [`MountGuard`].
pub(crate) fn mount_overlayfs(
    mount_dir: &Path,
    lower_dirs: &[&Path],
    upper_dir: &Path,
    scratch_dir: &Path,
) -> Result<MountGuard> {
    ensure_dir_is_empty(scratch_dir)?;
    ensure!(
        !lower_dirs.is_empty(),
        "Mounting overlayfs with zero lower directories is not supported"
    );

    let lowers_dir = scratch_dir.join("lowers");
    let work_dir = scratch_dir.join("work");

    let mut dir_builder = std::fs::DirBuilder::new();
    dir_builder.recursive(true).mode(0o755);

    for dir in [&lowers_dir, &work_dir] {
        dir_builder.create(dir)?;
    }

    // Bind-mount lower directories so we can refer to them in overlayfs options
    // in very short file paths because the maximum length of option strings for
    // mount(2) is constrained.
    let mut short_lower_dirs: Vec<(String, MountGuard)> = Vec::new();

    for (i, lower_dir) in lower_dirs.iter().enumerate() {
        let name = i.to_string();
        let path = lowers_dir.join(&name);

        dir_builder.create(&path)?;
        let guard = bind_mount(lower_dir, &path)?;

        short_lower_dirs.push((name, guard));
    }

    let runfiles = runfiles::Runfiles::create()?;
    let helper_path = runfiles
        .rlocation("cros/bazel/portage/bin/overlayfs_mount_helper/overlayfs_mount_helper")
        .canonicalize()?;

    // overlayfs supports up to 500 lower directories, but we often want to go beyond that.
    // We workaround it by stacking two overlayfs. This way, we can support up to 250,000 lower
    // directories, which should be sufficient. However note that Linux kernel supports stacking
    // overlayfs up to 2 layers, so it means that this will fail if lower directories are already
    // on overlayfs. Search the kernel code with FILESYSTEM_MAX_STACK_DEPTH for details.
    const MAX_LOWER_DIRS: usize = 500;

    ensure!(
        short_lower_dirs.len() <= MAX_LOWER_DIRS * MAX_LOWER_DIRS,
        "Too many lower directories ({})",
        short_lower_dirs.len()
    );

    if short_lower_dirs.len() > MAX_LOWER_DIRS {
        eprintln!(
            "WARNING: Stacking two overlayfs because there are too many lower directories \
            ({} > {}). This may fail to mount if any lower directory is already on overlayfs.",
            short_lower_dirs.len(),
            MAX_LOWER_DIRS,
        );

        let mut new_short_lower_dirs: Vec<(String, MountGuard)> = Vec::new();

        let chunks = short_lower_dirs.into_iter().chunks(MAX_LOWER_DIRS);
        for (i, chunk) in chunks.into_iter().enumerate() {
            let chunk: Vec<(String, MountGuard)> = chunk.into_iter().collect();
            if chunk.len() == 1 {
                // overlayfs fails to mount with no upper layer and exactly 1 lower layer, so
                // specially handle this case.
                let entry = chunk.into_iter().next().unwrap();
                new_short_lower_dirs.push(entry);
                continue;
            }

            let chunk_name = format!("c{}", i);
            let chunk_dir = lowers_dir.join(&chunk_name);
            dir_builder.create(&chunk_dir)?;

            let overlay_options = format!(
                "lowerdir={}",
                // Overlayfs option treats the first lower directory as the least lower
                // directory, while we order filesystem layers in the opposite order.
                chunk.iter().map(|(name, _guard)| name).rev().join(":")
            );

            // Mount overlayfs via overlayfs_mount_helper.
            // We don't call mount(2) directly because it requires us to change the
            // working directory of the current process, which introduces tricky issues
            // in multi-threaded programs, including unit tests.
            let status = Command::new(&helper_path)
                .arg(overlay_options)
                .arg(&chunk_dir)
                .current_dir(&lowers_dir)
                .status()?;
            ensure!(status.success(), "Failed to mount overlayfs: {:?}", status);

            new_short_lower_dirs.push((chunk_name, MountGuard::new(&chunk_dir)));

            // At this point bind-mounts for the lower directories are unmounted, but it's fine
            // because overlayfs holds references to those mounts internally.
        }

        short_lower_dirs = new_short_lower_dirs;
    }

    // b/295780293 - We enable xino because otherwise we don't get persistent inodes, even when
    // only writing to the upperdir.
    let overlay_options = format!(
        "upperdir={},workdir={},lowerdir={},xino=on",
        upper_dir.display(),
        work_dir.display(),
        // Overlayfs option treats the first lower directory as the least lower
        // directory, while we order filesystem layers in the opposite order.
        short_lower_dirs
            .iter()
            .map(|(name, _guard)| name)
            .rev()
            .join(":"),
    );

    // Mount overlayfs via overlayfs_mount_helper.
    // We don't call mount(2) directly because it requires us to change the
    // working directory of the current process, which introduces tricky issues
    // in multi-threaded programs, including unit tests.
    let status = Command::new(&helper_path)
        .arg(overlay_options)
        .arg(mount_dir)
        .current_dir(&lowers_dir)
        .status()?;
    ensure!(status.success(), "Failed to mount overlayfs: {:?}", status);
    let overlayfs_mount_guard = MountGuard::new(mount_dir);

    // At this point bind-mounts for the lower directories are unmounted, but it's fine because
    // overlayfs holds references to those mounts internally.
    Ok(overlayfs_mount_guard)
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    use anyhow::bail;
    use fileutil::SafeTempDir;

    use super::*;

    fn ensure_no_mount_under(dir: &Path) -> Result<()> {
        for line in BufReader::new(File::open("/proc/mounts")?).lines() {
            let line = line?;
            let path = Path::new(
                line.split(' ')
                    .nth(1)
                    .with_context(|| format!("Corrupted line in /proc/mounts: {}", &line))?,
            );
            if path.starts_with(dir) {
                bail!("Mount {} exists under {}", path.display(), dir.display())
            }
        }
        Ok(())
    }

    // Try mounting a minimal overlayfs with a single empty lower directory.
    #[test]
    fn minimal() -> Result<()> {
        let temp_dir = SafeTempDir::new()?;
        let temp_dir = temp_dir.path();

        let mount_dir = temp_dir.join("mount");
        let upper_dir = temp_dir.join("upper");
        let scratch_dir = temp_dir.join("scratch");
        let empty_dir = temp_dir.join("empty");
        for dir in [&mount_dir, &upper_dir, &scratch_dir, &empty_dir] {
            std::fs::create_dir(dir)?;
        }

        let mount_guard =
            mount_overlayfs(&mount_dir, &[empty_dir.as_path()], &upper_dir, &scratch_dir)?;

        // As soon as it finishes mounting, no mount points must be left outside
        // the overlayfs mount directory.
        ensure_no_mount_under(&upper_dir)?;
        ensure_no_mount_under(&scratch_dir)?;

        // The result is an empty directory.
        ensure_dir_is_empty(&mount_dir)?;

        // Unmount the overlayfs. Then only the empty directory should be left.
        drop(mount_guard);
        ensure_dir_is_empty(&mount_dir)?;
        ensure_no_mount_under(&mount_dir)?;

        Ok(())
    }

    // Try mounting an overlayfs with a few lower directories.
    #[test]
    fn simple() -> Result<()> {
        let temp_dir = SafeTempDir::new()?;
        let temp_dir = temp_dir.path();

        let mount_dir = temp_dir.join("mount");
        let upper_dir = temp_dir.join("upper");
        let lower1_dir = temp_dir.join("lower1");
        let lower2_dir = temp_dir.join("lower2");
        let scratch_dir = temp_dir.join("scratch");
        for dir in [
            &mount_dir,
            &upper_dir,
            &lower1_dir,
            &lower2_dir,
            &scratch_dir,
        ] {
            std::fs::create_dir(dir)?;
        }

        for file in [
            lower1_dir.join("A"),
            lower1_dir.join("B"),
            lower2_dir.join("C"),
        ] {
            File::create(file)?;
        }

        for dir in [lower1_dir.join("D"), lower2_dir.join("B")] {
            std::fs::create_dir(dir)?;
        }

        let mount_guard = mount_overlayfs(
            &mount_dir,
            &[lower1_dir.as_path(), lower2_dir.as_path()],
            &upper_dir,
            &scratch_dir,
        )?;

        // As soon as it finishes mounting, no mount points must be left outside
        // the overlayfs mount directory.
        ensure_no_mount_under(&upper_dir)?;
        ensure_no_mount_under(&lower1_dir)?;
        ensure_no_mount_under(&lower2_dir)?;
        ensure_no_mount_under(&scratch_dir)?;

        // From lower1/A.
        assert!(std::fs::metadata(mount_dir.join("A"))?.is_file());

        // lower1/B is a regular file, while lower2/B is a directory.
        // lower2/B takes precedence because it is listed later in the lower
        // directory list.
        assert!(std::fs::metadata(mount_dir.join("B"))?.is_dir());

        // From lower2/C.
        assert!(std::fs::metadata(mount_dir.join("C"))?.is_file());

        // From lower1/D.
        assert!(std::fs::metadata(mount_dir.join("D"))?.is_dir());

        // Unmount the overlayfs. Then only the empty directory is left.
        drop(mount_guard);
        ensure_dir_is_empty(&mount_dir)?;
        ensure_no_mount_under(&mount_dir)?;

        Ok(())
    }

    // Ensure mount points are not leaked on errors.
    #[test]
    fn no_mount_leak_on_errors() -> Result<()> {
        let temp_dir = SafeTempDir::new()?;
        let temp_dir = temp_dir.path();

        let mount_dir = temp_dir.join("mount");
        let upper_dir = temp_dir.join("upper");
        let lower1_dir = temp_dir.join("lower1");
        let lower2_dir = temp_dir.join("lower2");
        let scratch_dir = temp_dir.join("scratch");
        for dir in [
            &mount_dir,
            // Intentionally skip creating the upper directory.
            &lower1_dir,
            &lower2_dir,
            &scratch_dir,
        ] {
            std::fs::create_dir(dir)?;
        }

        // Mounting overlayfs should fail because the upper directory does not
        // exist.
        assert!(
            mount_overlayfs(
                &mount_dir,
                &[lower1_dir.as_path(), lower2_dir.as_path()],
                &upper_dir,
                &scratch_dir,
            )
            .is_err(),
            "mount_overlayfs must fail"
        );

        // The mount directory must have been cleaned up.
        ensure_dir_is_empty(&mount_dir)?;

        // No mount points must be left.
        ensure_no_mount_under(&mount_dir)?;
        ensure_no_mount_under(&upper_dir)?;
        ensure_no_mount_under(&lower1_dir)?;
        ensure_no_mount_under(&lower2_dir)?;
        ensure_no_mount_under(&scratch_dir)?;

        Ok(())
    }

    // Ensure we support any number of lower directories.
    #[test]
    fn many_lower_dirs() -> Result<()> {
        for n in [1, 2, 3, 498, 499, 500, 501, 502, 998, 999, 1000, 1001, 1002] {
            let temp_dir = SafeTempDir::new()?;
            let temp_dir = temp_dir.path();

            let mount_dir = temp_dir.join("mount");
            let upper_dir = temp_dir.join("upper");
            let lowers_dir = temp_dir.join("lowers");
            let scratch_dir = temp_dir.join("scratch");
            for dir in [&mount_dir, &upper_dir, &lowers_dir, &scratch_dir] {
                std::fs::create_dir(dir)?;
            }

            let mut lower_dirs: Vec<PathBuf> = Vec::new();
            for i in 0..n {
                let lower_dir = lowers_dir.join(i.to_string());
                std::fs::create_dir(&lower_dir)?;
                File::create(lower_dir.join(format!("{}.txt", i)))?;
                lower_dirs.push(lower_dir);
            }

            let mount_guard = mount_overlayfs(
                &mount_dir,
                &lower_dirs.iter().map(|d| d.as_path()).collect_vec(),
                &upper_dir,
                &scratch_dir,
            )?;

            // As soon as it finishes mounting, no mount points must be left outside
            // the overlayfs mount directory.
            ensure_no_mount_under(&upper_dir)?;
            ensure_no_mount_under(&lowers_dir)?;
            ensure_no_mount_under(&scratch_dir)?;

            // The result is a directory with exactly `n` files.
            assert_eq!(std::fs::read_dir(&mount_dir)?.count(), n);

            // Unmount the overlayfs. Then only the empty directory should be left.
            drop(mount_guard);
            ensure_dir_is_empty(&mount_dir)?;
            ensure_no_mount_under(&mount_dir)?;
        }

        Ok(())
    }
}
