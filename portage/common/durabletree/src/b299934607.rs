// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    os::unix::prelude::MetadataExt,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use anyhow::{bail, Result};
use itertools::Itertools;
use walkdir::WalkDir;

/// Waits until Bazel finishes calling chmod on the durable tree.
///
/// This is a workaround for b/299934607 where Bazel may start actions before Bazel finishes
/// updating directory permissions.
pub(crate) fn wait_chmods(root_dir: &Path, timeout: Duration) -> Result<()> {
    let mut remaining_dirs: Vec<PathBuf> = Vec::new();

    // Scan directories to build a list of directories to watch.
    for entry in WalkDir::new(root_dir)
        .follow_links(true) // follow symlinks in execroot
        .sort_by_file_name()
    {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() && metadata.mode() & 0o777 != 0o555 {
            remaining_dirs.push(entry.path().to_path_buf());
        }
    }

    let start_time = Instant::now();
    while !remaining_dirs.is_empty() {
        if start_time.elapsed() >= timeout {
            bail!(
                "Durable tree's directories have wrong permissions after waiting {}s \
                (b/299934607):\n{}",
                timeout.as_secs(),
                remaining_dirs
                    .into_iter()
                    .map(|p| p.to_string_lossy().into_owned())
                    .join("\n")
            );
        }

        let mut next_remaining_dirs: Vec<PathBuf> = Vec::new();
        for path in remaining_dirs {
            let metadata = path.metadata()?;
            if metadata.mode() & 0o777 != 0o555 {
                next_remaining_dirs.push(path);
            }
        }
        remaining_dirs = next_remaining_dirs;

        std::thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs::Permissions, os::unix::prelude::PermissionsExt};

    use fileutil::SafeTempDir;

    use super::*;

    #[test]
    fn test_wait_chmods_immediate_success() -> Result<()> {
        let root_dir = SafeTempDir::new()?;
        let root_dir = root_dir.path();

        std::fs::set_permissions(root_dir, Permissions::from_mode(0o555))?;

        let subdirs = [
            root_dir.join("aaa"),
            root_dir.join("bbb"),
            root_dir.join("ccc"),
        ];

        for subdir in subdirs.iter() {
            std::fs::create_dir(subdir)?;
            std::fs::set_permissions(subdir, Permissions::from_mode(0o555))?;
        }

        wait_chmods(root_dir, Duration::from_millis(100))?;

        Ok(())
    }

    #[test]
    fn test_wait_chmods_failure() -> Result<()> {
        let root_dir = SafeTempDir::new()?;
        let root_dir = root_dir.path();

        std::fs::set_permissions(root_dir, Permissions::from_mode(0o555))?;

        let subdirs = [
            root_dir.join("aaa"),
            root_dir.join("bbb"),
            root_dir.join("ccc"),
        ];

        for subdir in subdirs.iter() {
            std::fs::create_dir(subdir)?;
            std::fs::set_permissions(subdir, Permissions::from_mode(0o555))?;
        }

        // Set a wrong permissions.
        std::fs::set_permissions(&subdirs[1], Permissions::from_mode(0o777))?;

        assert!(wait_chmods(root_dir, Duration::from_millis(100)).is_err());

        Ok(())
    }

    #[test]
    fn test_wait_chmods_async_chmod() -> Result<()> {
        let root_dir = SafeTempDir::new()?;
        let root_dir = root_dir.path();

        std::fs::set_permissions(root_dir, Permissions::from_mode(0o555))?;

        let subdirs = [
            root_dir.join("aaa"),
            root_dir.join("bbb"),
            root_dir.join("ccc"),
        ];

        for subdir in subdirs.iter() {
            std::fs::create_dir(subdir)?;
            std::fs::set_permissions(subdir, Permissions::from_mode(0o555))?;
        }

        // Set a wrong permission.
        std::fs::set_permissions(&subdirs[1], Permissions::from_mode(0o777))?;

        std::thread::scope(|s| -> Result<()> {
            // Set a correct permission asynchronously.
            let handle = s.spawn(|| -> Result<()> {
                std::thread::sleep(Duration::from_millis(300));
                std::fs::set_permissions(&subdirs[1], Permissions::from_mode(0o555))?;
                Ok(())
            });

            wait_chmods(root_dir, Duration::from_secs(10))?;
            handle.join().expect("Thread panicked")?;
            Ok(())
        })?;

        Ok(())
    }
}
