// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, ensure, Context, Result};
use std::path::{Component, Path, PathBuf};

/// Joins an absolute `path` to `root`.
pub fn join_absolute(root: &Path, path: &Path) -> Result<PathBuf> {
    Ok(root.join(
        path.strip_prefix("/")
            .with_context(|| format!("path {} is not absolute", path.display()))?,
    ))
}
/// Removes ./ and as many ../ operators from the path as possible while
/// respecting symlinks.
///
/// The `path` must exist otherwise an error is returned.
///
/// If a ../ is encountered, the parent will be popped off the path if it is
/// a directory.
pub fn clean_path(path: &Path) -> Result<PathBuf> {
    let mut parts = PathBuf::new();

    ensure!(path.is_absolute(), "Expected an absolute path");

    for component in path.components() {
        match component {
            Component::RootDir => {
                parts.push(Component::RootDir);
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if parts.parent().is_none() {
                    // /../ => /
                    continue;
                }

                if parts.ends_with(Component::ParentDir) {
                    // Never pop a ../ because the parent is a symlink.
                    parts.push(Component::ParentDir)
                } else if parts.symlink_metadata()?.is_dir() {
                    parts.pop();
                } else {
                    // Parent is a symlink, so keep the ../.
                    parts.push(Component::ParentDir)
                }
            }
            Component::Normal(part) => {
                parts.push(part);
            }
            Component::Prefix(_) => bail!("Windows paths are not supported"),
        }
    }

    Ok(parts)
}

#[cfg(test)]
mod tests {
    use crate::testutils::write_files;

    use super::*;
    use std::os::unix::fs;

    #[test]
    fn test_simple_clean_path() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(
            dir,
            [
                (
                    "profiles/default/linux/x86/10.0/chromeos/parent",
                    "../../../../../targets/chromeos",
                ),
                ("profiles/targets/chromeos/parent", "../../features/selinux"),
            ],
        )?;

        let orig = dir.join(
            "profiles/default/linux/x86/10.0/chromeos/../../../../../targets/chromeos/parent",
        );

        let clean = clean_path(&orig)?;

        assert_eq!(dir.join("profiles/targets/chromeos/parent"), clean);

        assert!(clean.try_exists()?);

        Ok(())
    }

    #[test]
    fn test_symlink_clean_path() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(
            dir,
            [
                (
                    "profiles/default/linux/x86/10.0/chromeos/parent",
                    "../../../../../targets/chromeos",
                ),
                ("profiles/targets/chromeos/parent", "../../features/selinux"),
            ],
        )?;

        fs::symlink("10.0", dir.join("profiles/default/linux/x86/11.0"))?;
        fs::symlink("linux", dir.join("profiles/default/bsd"))?;

        let orig = dir
            .join("profiles/default/bsd/x86/../../linux/x86/11.0/chromeos/../../../../../targets/chromeos/parent");

        let clean = clean_path(&orig)?;

        assert_eq!(
            dir.join("profiles/default/bsd/../linux/x86/11.0/../../../../targets/chromeos/parent"),
            clean
        );

        assert!(clean.try_exists()?);

        Ok(())
    }

    #[test]
    fn test_root() -> Result<()> {
        let orig = Path::new("/../../tmp");

        let clean = clean_path(orig)?;

        assert_eq!(Path::new("/tmp"), clean);

        Ok(())
    }
}
