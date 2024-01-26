// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, ensure, Context, Result};

/// The name of the environment variable controlling whether to regenerate
/// golden data.
const REGENERATE_VAR_NAME: &str = "ALCHEMY_REGENERATE_GOLDEN";

fn should_regenerate() -> bool {
    std::env::var(REGENERATE_VAR_NAME).unwrap_or_default() != ""
}

// Renames output files as required to ensure that bazel doesn't interpret them as bazel packages.
fn rename_bazel_special_files(dir: &Path) -> std::io::Result<()> {
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.file_name() == Some(std::ffi::OsStr::new("BUILD.bazel")) {
            std::fs::rename(path, path.with_file_name("BUILD.golden.bazel"))?;
        }
    }
    Ok(())
}

fn compute_real_golden_path(golden: &Path) -> Result<PathBuf> {
    ensure!(
        golden.is_relative(),
        "Golden path must be relative to the workspace root! \
        See the description of compare_with_golden_data."
    );
    Ok(if should_regenerate() {
        // When regenerating under Bazel, writing to the runfiles root would just write to the
        // sandbox.
        crate::workspace_root()
            .context("Unable to write to the workspace from a test since it's in a sandbox.")?
    } else {
        crate::runfiles_root()?
    }
    .join(golden))
}

/// Compares contents of the two directories and returns an error if there is
/// any mismatch.
///
/// When this function is called from tests running under Bazel, the golden
/// directory path must be a relative path. Bazel sets up the initial current
/// directory for tests to the associated runfiles directory, so you should be
/// able to refer to Git-committed golden data simply in relative paths.
///
/// # Updating golden data
///
/// This function updates the golden data with the output data if the
/// environment variable `ALCHEMY_REGENERATE_GOLDEN` is set to a non-empty
/// value.
///
/// When tests are run under Bazel with `bazel test`, it is impossible to
/// automatically update golden data due to the Bazel sandbox. Use `bazel run`
/// instead, e.g.
///
/// ```sh
/// ALCHEMY_REGENERATE_GOLDEN=1 bazel run :foo_test
/// ```
///
/// When tests are run under Cargo, you can just pass the environment variable
/// to `cargo test`.
///
/// ```sh
/// ALCHEMY_REGENERATE_GOLDEN=1 cargo test
/// ```
pub fn compare_with_golden_data(output: &Path, golden: &Path) -> Result<()> {
    let real_golden = &compute_real_golden_path(golden)?;

    if output.is_dir() {
        rename_bazel_special_files(output)?;
    }

    if should_regenerate() {
        if real_golden.is_dir() {
            std::fs::remove_dir_all(real_golden)?;
        } else if real_golden.is_file() {
            std::fs::remove_file(real_golden)?;
        } else {
            ensure!(!real_golden.try_exists()?, "Unknown file type");
        }
        let status = Command::new("cp")
            .args(["--recursive", "--dereference", "--"])
            .arg(output)
            .arg(real_golden)
            .status()?;
        ensure!(
            status.success(),
            "Failed to update golden data: {:?}",
            status
        );
    } else {
        let bazel_target = std::env::var("TEST_TARGET").ok();
        if let Some(ref bazel_target) = bazel_target {
            ensure!(
                real_golden.try_exists()?,
                "The golden directory {real_golden:?} doesn't exist. Maybe you had a typo, maybe \
                you forgot to include it in the data attribute, or maybe you just need to \
                regenerate the golden data.\n\
                To regenerate them, run 'ALCHEMY_REGENERATE_GOLDEN=1 bazel run {bazel_target}'"
            );
        }
        let status = Command::new("diff")
            .args(["-Naru", "--"])
            .arg(real_golden)
            .arg(output)
            .status()?;
        if !status.success() {
            // Print a friendly instruction if we're running under Bazel.
            if let Some(bazel_target) = bazel_target {
                bail!(
                    "Found mismatch with golden data; \
                    consider regenerating them with: ALCHEMY_REGENERATE_GOLDEN=1 bazel run {}",
                    bazel_target,
                )
            } else {
                bail!("Found mismatch with golden data");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::OsString,
        sync::{Mutex, MutexGuard},
    };

    use once_cell::sync::OnceCell;
    use tempfile::{NamedTempFile, TempDir};

    use super::*;

    const GOLDEN_DIR: &str = "bazel/portage/common/testutil/testdata/golden";

    /// Used by [`RegenVarLock`] to prevents multiple tests from running in
    /// parallel.
    static MUTEX: OnceCell<Mutex<()>> = OnceCell::new();

    /// Sets the environment variable [`REGENERATE_VAR_NAME`] while preventing
    /// multiple tests from running in parallel.
    struct RegenVarLock<'a> {
        _lock: MutexGuard<'a, ()>,
        original_value: Option<OsString>,
    }

    impl RegenVarLock<'_> {
        /// Sets the environment variable [`REGENERATE_VAR_NAME`] to the
        /// specified value, and acquires a lock to prevent multiple tests from
        /// running in parallel.
        ///
        /// On dropping the returned object, the environment variable is reset
        /// to the original value and the lock is released.
        pub fn acquire(regenerate: bool) -> Self {
            let lock = MUTEX.get_or_init(|| Mutex::new(())).lock().unwrap();

            let original_value = std::env::var_os(REGENERATE_VAR_NAME);
            let new_value = if regenerate { "1" } else { "" };
            std::env::set_var(REGENERATE_VAR_NAME, new_value);
            Self {
                _lock: lock,
                original_value,
            }
        }
    }

    impl Drop for RegenVarLock<'_> {
        fn drop(&mut self) {
            match &self.original_value {
                None => {
                    std::env::remove_var(REGENERATE_VAR_NAME);
                }
                Some(original_value) => {
                    std::env::set_var(REGENERATE_VAR_NAME, original_value);
                }
            }
        }
    }

    #[test]
    fn test_compare_dirs_success() -> Result<()> {
        let _lock = RegenVarLock::acquire(false);

        let output_dir = TempDir::new()?;
        let output_dir = output_dir.path();

        std::fs::write(output_dir.join("a.txt"), "aaa\n")?;
        std::fs::write(output_dir.join("b.txt"), "bbb\n")?;
        std::fs::create_dir(output_dir.join("d"))?;
        std::fs::write(output_dir.join("d/c.txt"), "ccc\n")?;

        compare_with_golden_data(output_dir, Path::new(GOLDEN_DIR))?;
        Ok(())
    }

    #[test]
    fn test_compare_dirs_failure() -> Result<()> {
        let _lock = RegenVarLock::acquire(false);

        let output_dir = TempDir::new()?;
        let output_dir = output_dir.path();

        std::fs::write(output_dir.join("a.txt"), "aaa\n")?;
        std::fs::write(output_dir.join("b.txt"), "xxx\n")?;
        std::fs::create_dir(output_dir.join("d"))?;
        std::fs::write(output_dir.join("d/c.txt"), "ccc\n")?;

        assert!(compare_with_golden_data(output_dir, Path::new(GOLDEN_DIR)).is_err());
        Ok(())
    }

    #[test]
    fn test_compare_files_success() -> Result<()> {
        let _lock = RegenVarLock::acquire(false);

        let file = NamedTempFile::new()?;
        std::fs::write(file.path(), "aaa\n")?;

        compare_with_golden_data(file.path(), &Path::new(GOLDEN_DIR).join("a.txt"))?;
        Ok(())
    }

    #[test]
    fn test_compare_files_failure() -> Result<()> {
        let _lock = RegenVarLock::acquire(false);

        let file = NamedTempFile::new()?;
        std::fs::write(file.path(), "xxx\n")?;

        assert!(
            compare_with_golden_data(file.path(), &Path::new(GOLDEN_DIR).join("a.txt")).is_err()
        );
        Ok(())
    }

    // TODO: Write a test that regenerates golden data. It is not trivial to
    // write one because of Bazel sandbox.
}
