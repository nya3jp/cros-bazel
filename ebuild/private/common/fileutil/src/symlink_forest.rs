// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::path::{Path, PathBuf};

use anyhow::{ensure, Context, Result};
use walkdir::WalkDir;

/// Resolves a file path to a Bazel action input to its actual file path where
/// no symlinks are involved.
///
/// Bazel often creates a symlink forest, consisting of directories and symlinks
/// to regular files, to create execroot without copying files. Symlink forests
/// usually work just like their real files, but they can cause problems when
/// being accessed from different mount namespaces. You can use this function to
/// get an actual file path of a symlink forest to work around the issue.
///
/// The given input path must point to a Bazel action input file or directory.
/// Otherwise this function may return wrong results when it is a directory
/// containing symlinks.
pub fn resolve_symlink_forest(input_path: &Path) -> Result<PathBuf> {
    let metadata = std::fs::symlink_metadata(input_path)
        .with_context(|| format!("Failed to get metadata: {}", input_path.display()))?;
    if metadata.is_symlink() {
        let target = std::fs::read_link(input_path)?;
        let real_target = input_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(target);
        return Ok(real_target.canonicalize()?);
    }
    if metadata.is_file() {
        return Ok(input_path.canonicalize()?);
    }
    ensure!(
        metadata.is_dir(),
        "Unknown file type: {:?}",
        metadata.file_type()
    );

    for entry in WalkDir::new(input_path) {
        let entry = entry?;
        let metadata = entry.metadata()?;

        if metadata.is_symlink() {
            let target = std::fs::read_link(entry.path())?;
            let real_target = entry
                .path()
                .parent()
                .unwrap_or_else(|| Path::new(""))
                .join(target);

            let mut resolved: &Path = &real_target;
            let relative_path = entry.path().strip_prefix(input_path)?;
            for _ in relative_path.components() {
                resolved = resolved
                    .parent()
                    .context("Symlink target should have parent")?;
            }

            return Ok(resolved.canonicalize()?);
        } else if metadata.is_file() {
            return Ok(input_path.canonicalize()?);
        }
    }

    // In the case that the directory is empty, we still want the returned path to
    // be valid.
    Ok(input_path.canonicalize()?)
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{create_dir_all, File},
        os::unix::fs::symlink,
    };

    use crate::SafeTempDir;

    use super::*;

    #[test]
    fn test_resolve_symlink_forest() -> Result<()> {
        let actual_dir = SafeTempDir::new()?;
        let actual_dir = actual_dir.path();
        create_dir_all(actual_dir.join("a/b/c"))?;
        File::create(actual_dir.join("a/b/c/hello.txt"))?;

        let forest_dir = SafeTempDir::new()?;
        let forest_dir = forest_dir.path();
        create_dir_all(forest_dir.join("a/b/c"))?;
        symlink(
            actual_dir.join("a/b/c/hello.txt"),
            forest_dir.join("a/b/c/hello.txt"),
        )?;

        assert_eq!(resolve_symlink_forest(forest_dir)?, actual_dir.to_owned());
        Ok(())
    }

    #[test]
    fn test_resolve_symlink_forest_empty_dir() -> Result<()> {
        let forest_dir = SafeTempDir::new()?;
        let forest_dir = forest_dir.path();

        assert_eq!(resolve_symlink_forest(forest_dir)?, forest_dir.to_owned());
        Ok(())
    }

    #[test]
    fn test_resolve_symlink_forest_real_dir() -> Result<()> {
        let actual_dir = SafeTempDir::new()?;
        let actual_dir = actual_dir.path();
        create_dir_all(actual_dir.join("a/b/c"))?;
        File::create(actual_dir.join("a/b/c/hello.txt"))?;

        assert_eq!(resolve_symlink_forest(actual_dir)?, actual_dir.to_owned());
        Ok(())
    }

    #[test]
    fn test_resolve_symlink_forest_real_file() -> Result<()> {
        let actual_dir = SafeTempDir::new()?;
        let actual_dir = actual_dir.path();
        let actual_file_path = actual_dir.join("hello.txt");
        File::create(&actual_file_path)?;

        assert_eq!(resolve_symlink_forest(&actual_file_path)?, actual_file_path);
        Ok(())
    }

    #[test]
    fn test_resolve_symlink_forest_direct_symlink() -> Result<()> {
        let actual_dir = SafeTempDir::new()?;
        let actual_dir = actual_dir.path();
        File::create(actual_dir.join("hello.txt"))?;

        let temp_dir = SafeTempDir::new()?;
        let temp_dir = temp_dir.path();
        let symlink_path = temp_dir.join("symlink");
        symlink(actual_dir, &symlink_path)?;

        assert_eq!(
            resolve_symlink_forest(&symlink_path)?,
            actual_dir.to_owned()
        );
        Ok(())
    }
}
