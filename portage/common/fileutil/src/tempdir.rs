// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

use anyhow::Result;
use lazy_static::lazy_static;
use tracing::info_span;

use crate::remove_dir_all_with_chmod;

lazy_static! {
    static ref DEFAULT_PREFIX: OsString = {
        let current_exe = std::env::current_exe().unwrap_or_default();
        let current_program_name = current_exe
            .file_name()
            .unwrap_or(OsStr::new("__unknown__"))
            .to_string_lossy();
        format!("alchemy.{}.", current_program_name).into()
    };
}

/// Safer version of [`tempfile::TempDir`].
///
/// Notable differences to [`tempfile::TempDir`] are:
/// - Directory names are prefixed with the current program name by default
///   so that it's easier to debug temporary directory issues.
/// - Uses [`remove_dir_all_with_chmod`] to remove files that could not be
///   removed simply by [`std::fs::remove_dir_all`].
pub struct SafeTempDir {
    dir: Option<PathBuf>,
}

impl SafeTempDir {
    /// Creates a new temporary directory using the default configuration.
    ///
    /// Use [`SafeTempDirBuilder`] instead if you want more configurations.
    pub fn new() -> Result<Self> {
        SafeTempDirBuilder::new().build()
    }

    /// Creates a [`SafeTempDir`] by taking the ownership of an existing
    /// directory.
    pub fn take(dir: &Path) -> Self {
        Self {
            dir: Some(dir.to_path_buf()),
        }
    }

    /// Returns the path to the temporary directory.
    pub fn path(&self) -> &Path {
        self.dir.as_ref().unwrap()
    }

    /// Converts [`SafeTempDir`] into [`PathBuf`]. After calling this function,
    /// it is the caller's responsibility to remove the directory after use.
    pub fn into_path(mut self) -> PathBuf {
        self.dir.take().unwrap()
    }
}

impl Drop for SafeTempDir {
    fn drop(&mut self) {
        if let Some(dir) = &self.dir {
            let _span = info_span!("SafeTempDir::drop", dir = ?dir).entered();
            remove_dir_all_with_chmod(dir).expect("Failed to remove temporary directory");
        }
    }
}

pub struct SafeTempDirBuilder<'prefix, 'suffix> {
    builder: tempfile::Builder<'prefix, 'suffix>,
    base_dir: PathBuf,
}

impl<'prefix, 'suffix> SafeTempDirBuilder<'prefix, 'suffix> {
    /// Creates a new builder for [`SafeTempDir`].
    pub fn new() -> Self {
        let mut builder = tempfile::Builder::new();
        builder.prefix(&*DEFAULT_PREFIX);
        let base_dir = std::env::temp_dir();
        Self { builder, base_dir }
    }

    /// Sets the base directory where a new temporary directory is created.
    pub fn base_dir(self, dir: &Path) -> Self {
        Self {
            base_dir: dir.to_owned(),
            ..self
        }
    }

    /// Sets a custom file name prefix.
    pub fn prefix<S: AsRef<OsStr> + ?Sized>(mut self, prefix: &'prefix S) -> Self {
        self.builder.prefix(prefix);
        self
    }

    /// Sets a custom file name suffix.
    pub fn suffix<S: AsRef<OsStr> + ?Sized>(mut self, suffix: &'suffix S) -> Self {
        self.builder.suffix(suffix);
        self
    }

    /// Builds [`SafeTempDir`].
    pub fn build(self) -> Result<SafeTempDir> {
        let dir = self.builder.tempdir_in(self.base_dir)?;
        Ok(SafeTempDir::take(&dir.into_path()))
    }
}

impl Default for SafeTempDirBuilder<'_, '_> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{create_dir, set_permissions},
        os::unix::prelude::PermissionsExt,
    };

    use super::*;

    #[test]
    fn test_safe_temp_dir_deletes_inaccessible_dirs() -> Result<()> {
        let temp_dir = SafeTempDir::new()?;
        let path = temp_dir.path().to_owned();

        // Create an unaccessible directory.
        let bad_dir = path.join("bad");
        create_dir(&bad_dir)?;
        set_permissions(&bad_dir, PermissionsExt::from_mode(0o0))?;

        drop(temp_dir);

        assert!(!bad_dir.try_exists()?);

        Ok(())
    }

    #[test]
    fn test_safe_temp_dir_is_pretty_named() -> Result<()> {
        let temp_dir = SafeTempDir::new()?;
        let temp_dir_name = temp_dir.path().file_name().unwrap().to_string_lossy();
        assert!(
            temp_dir_name.starts_with("alchemy.fileutil"),
            "temp_dir_name = {}",
            temp_dir_name
        );
        Ok(())
    }

    #[test]
    fn test_safe_temp_dir_with_base_dir() -> Result<()> {
        let temp_dir1 = SafeTempDir::new()?;
        let temp_dir2 = SafeTempDirBuilder::new()
            .base_dir(temp_dir1.path())
            .build()?;
        assert!(temp_dir2.path().starts_with(temp_dir1.path()));
        Ok(())
    }

    #[test]
    fn test_safe_temp_dir_with_custom_prefix_suffix() -> Result<()> {
        let temp_dir = SafeTempDirBuilder::new()
            .prefix("foo.")
            .suffix(".bar")
            .build()?;
        let temp_dir_name = temp_dir.path().file_name().unwrap().to_string_lossy();
        assert!(
            temp_dir_name.starts_with("foo.") && temp_dir_name.ends_with(".bar"),
            "Directory name: {}",
            temp_dir_name
        );
        Ok(())
    }

    #[test]
    fn test_safe_temp_dir_take() -> Result<()> {
        let temp_dir = SafeTempDir::new()?;
        let path = temp_dir.path().to_owned();

        let temp_dir = temp_dir.into_path();
        let temp_dir = SafeTempDir::take(&temp_dir);

        assert!(path.try_exists()?);
        drop(temp_dir);
        assert!(!path.try_exists()?);

        Ok(())
    }
}
