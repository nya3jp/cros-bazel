// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use binarypackage::BinaryPackage;
use clap::Parser;
use cliutil::cli_main;
use container::enter_mount_namespace;
use durabletree::DurableTree;
use std::{
    collections::HashMap,
    ffi::OsString,
    io::ErrorKind,
    os::unix::prelude::MetadataExt,
    path::{Path, PathBuf},
    process::ExitCode,
};
use vdb::{create_initial_vdb, create_sparse_vdb, generate_vdb_contents, get_vdb_dir};
use walkdir::WalkDir;

/// Unpacks a binary package file to generate an installed image that can be
/// mounted as an overlayfs layer.
#[derive(Parser, Debug)]
struct Cli {
    /// Input binary package file.
    #[arg(long)]
    input_binary_package: PathBuf,

    /// Output directory where the installed image is saved as a durable tree.
    #[arg(long)]
    output_directory: PathBuf,

    /// Directory prefix to add to the output image files.
    // Note: This is not `PathBuf` because the default parser doesn't allow
    // empty paths.
    #[arg(long)]
    image_prefix: String,

    /// Directory prefix to add to the output VDB directory.
    // Note: This is not `PathBuf` because the default parser doesn't allow
    // empty paths.
    #[arg(long)]
    vdb_prefix: String,

    /// Indicates that the package is for the host.
    #[arg(long)]
    host: bool,

    /// Omit most of the metadata from the vdb entry.
    ///
    /// Most of the vdb is not needed after a package has been installed.
    #[arg(long)]
    sparse_vdb: bool,
}

fn read_xattrs(path: &Path) -> Result<HashMap<OsString, Vec<u8>>> {
    let mut xattrs: HashMap<OsString, Vec<u8>> = HashMap::new();
    for name in xattr::list(path).context("Failed to list xattrs")? {
        let value = xattr::get(path, &name)?.unwrap_or_default();
        xattrs.insert(name, value);
    }
    Ok(xattrs)
}

fn ensure_directories_equivalent(source_path: &Path, target_path: &Path) -> Result<()> {
    let source_metadata = source_path
        .symlink_metadata()
        .with_context(|| format!("Failed to stat {}", source_path.display()))?;
    let target_metadata = target_path
        .symlink_metadata()
        .with_context(|| format!("Failed to stat {}", target_path.display()))?;
    if source_metadata.mode() != target_metadata.mode() {
        bail!(
            "Failed to merge two directories: inconsistent modes: 0{:o} vs. 0{:o}: {} and {}",
            source_metadata.mode(),
            target_metadata.mode(),
            source_path.display(),
            target_path.display()
        );
    }

    let source_xattrs = read_xattrs(source_path)
        .with_context(|| format!("Failed to read xattrs: {}", source_path.display()))?;
    let target_xattrs = read_xattrs(target_path)
        .with_context(|| format!("Failed to read xattrs: {}", target_path.display()))?;
    if source_xattrs != target_xattrs {
        bail!(
            "Failed to merge two directories: inconsistent xattrs: {} and {}",
            source_path.display(),
            target_path.display()
        );
    }

    Ok(())
}

/// Merges a source directory to a target directory.
///
/// Files under the source root directory are moved to the target root directory. The source root
/// directory must exist initially, and it will be removed upon success.
///
/// Here is the logic to merge two root directories, applied recursively from top to bottom:
/// - If a file under the source root directory is missing in the target root directory, it is
///   simply moved to the new location.
/// - If a file under the source root directory has a corresponding file in the target root
///   directory:
///   - If the two files are directories with equivalent metadata, they are kept as-is.
///     Two directories are considered to have equivalent metadata if they have the same
///     permissions and xattrs (which may contain ownership info recorded with fakefs).
///   - In all other cases, it is an error.
///
/// The logic applies to the root directory as well, so it is fine even if the target root directory
/// does not exist initially. However, the parent directory of the target root directory must exist.
fn merge_directories(source_root: &Path, target_root: &Path) -> Result<()> {
    let mut walk = WalkDir::new(source_root).sort_by_file_name().into_iter();
    // We can't use for-loops with `WalkDir::skip_current_dir`.
    loop {
        let source_entry = match walk.next() {
            None => break,
            Some(Ok(entry)) => entry,
            Some(Err(err)) => return Err(err.into()),
        };

        let source_path = source_entry.path();
        let target_path = &target_root.join(source_path.strip_prefix(source_root)?);

        let source_type = source_entry.file_type();
        let target_type = match target_path.symlink_metadata() {
            Ok(metadata) => metadata.file_type(),
            Err(err) if err.kind() == ErrorKind::NotFound => {
                std::fs::rename(source_path, target_path).with_context(|| {
                    format!(
                        "Failed to rename {} to {}",
                        source_path.display(),
                        target_path.display()
                    )
                })?;
                // Skip the directory since we've moved the entire tree.
                walk.skip_current_dir();
                continue;
            }
            Err(err) => {
                return Err(err).context(format!("Failed to stat {}", target_path.display()))
            }
        };

        if source_type.is_dir() && target_type.is_dir() {
            ensure_directories_equivalent(source_path, target_path)?;
        } else {
            bail!(
                "Failed to merge two directories: conflicting non-directory files: \
                {} ({:?}) and {} ({:?})",
                source_path.display(),
                source_type,
                target_path.display(),
                target_type,
            );
        }
    }

    // Remove remaining directories, if any.
    match std::fs::remove_dir_all(source_root) {
        Ok(()) => {}
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => return Err(err).context(format!("Failed to remove {}", source_root.display())),
    }

    Ok(())
}

fn do_main() -> Result<()> {
    let args = Cli::try_parse()?;

    let image_dir = args.output_directory.join(&args.image_prefix);
    std::fs::create_dir_all(&image_dir)?;

    let mut binary_package = BinaryPackage::open(&args.input_binary_package)?;
    binary_package.extract_image(&image_dir, true)?;

    let mut contents: Vec<u8> = Vec::new();
    generate_vdb_contents(&mut contents, &image_dir)?;

    let vdb_dir = get_vdb_dir(
        &args.output_directory.join(&args.vdb_prefix),
        binary_package.category_pf(),
    );

    // Once a package has been installed, we no longer need a full vdb entry.
    if args.sparse_vdb {
        create_sparse_vdb(&vdb_dir, &binary_package)?;
    } else {
        create_initial_vdb(&vdb_dir, &binary_package)?;
    }

    // We write the CONTENTS file regardless of sparse_vdb because it
    // doesn't result in unnecessary cache busting (since it's derived from the
    // input files), and it's also used by the find-missing-deps.sh hook.
    std::fs::write(vdb_dir.join("CONTENTS"), contents)?;

    if args.host {
        // HACK: Rename directories that collide with well-known symlinks.
        //
        // The host profile sets SYMLINK_LIB=yes, which causes sys-libs/glibc to
        // create symlinks /lib -> /lib64 and /usr/lib -> /usr/lib64. Those
        // symlinks are problematic when we use overlayfs to simulate package
        // installation because symlinks are replaced with regular directories
        // when a package contains /lib or /usr/lib as regular directories.
        // Until we set SYMLINK_LIB=no everywhere (crbug.com/360346), we work
        // around the issue by simply renaming directories.
        for (source, target) in [("lib", "lib64"), ("usr/lib", "usr/lib64")] {
            let source = image_dir.join(source);
            let target = image_dir.join(target);

            let source_metadata = match source.symlink_metadata() {
                Err(err) if err.kind() == ErrorKind::NotFound => continue,
                other => other?,
            };
            if source_metadata.is_dir() {
                merge_directories(&source, &target).with_context(|| {
                    format!(
                        "Failed to rename {} to {}",
                        source.display(),
                        target.display()
                    )
                })?;
            }
        }
    }

    DurableTree::convert(&args.output_directory)?;

    Ok(())
}

fn main() -> ExitCode {
    // We want CAP_DAC_OVERRIDE to scan read-protected directories on generating CONTENTS.
    enter_mount_namespace().expect("Failed to enter a mount namespace");
    cli_main(do_main, Default::default())
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{File, Permissions},
        os::unix::prelude::PermissionsExt,
    };

    use tempfile::TempDir;

    use super::*;

    // Tests the case where the target directory does not exist.
    #[test]
    fn test_merge_directories_target_missing() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let source_dir = &temp_dir.path().join("source");
        let target_dir = &temp_dir.path().join("target");

        std::fs::create_dir_all(source_dir.join("dir/aaa"))?;

        merge_directories(source_dir, target_dir)?;

        assert!(!source_dir.try_exists()?);
        assert!(target_dir.join("dir/aaa").is_dir());
        Ok(())
    }

    // Tests the case where the target directory does not have collisions.
    #[test]
    fn test_merge_directories_no_collisions() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let source_dir = &temp_dir.path().join("source");
        let target_dir = &temp_dir.path().join("target");

        std::fs::create_dir_all(source_dir.join("dir/aaa"))?;
        std::fs::create_dir_all(target_dir.join("dir/bbb"))?;

        merge_directories(source_dir, target_dir)?;

        assert!(!source_dir.try_exists()?);
        assert!(target_dir.join("dir/aaa").is_dir());
        assert!(target_dir.join("dir/bbb").is_dir());
        Ok(())
    }

    // Tests the case where the target directory have identical directories.
    #[test]
    fn test_merge_directories_identical_directories() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let source_dir = &temp_dir.path().join("source");
        let target_dir = &temp_dir.path().join("target");

        std::fs::create_dir_all(source_dir.join("dir/aaa"))?;
        std::fs::create_dir_all(target_dir.join("dir/aaa"))?;

        merge_directories(source_dir, target_dir)?;

        assert!(!source_dir.try_exists()?);
        assert!(target_dir.join("dir/aaa").is_dir());
        Ok(())
    }

    // Tests the case where the target directory have directories with inconsistent permissions.
    #[test]
    fn test_merge_directories_inconsistent_permissions() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let source_dir = &temp_dir.path().join("source");
        let target_dir = &temp_dir.path().join("target");

        std::fs::create_dir_all(source_dir.join("dir/aaa"))?;
        std::fs::create_dir_all(target_dir.join("dir/aaa"))?;
        std::fs::set_permissions(source_dir.join("dir/aaa"), Permissions::from_mode(0o705))?;

        assert!(merge_directories(source_dir, target_dir).is_err());
        Ok(())
    }

    // Tests the case where the target directory have directories with inconsistent xattrs.
    #[test]
    fn test_merge_directories_inconsistent_xattrs() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let source_dir = &temp_dir.path().join("source");
        let target_dir = &temp_dir.path().join("target");

        std::fs::create_dir_all(source_dir.join("dir/aaa"))?;
        std::fs::create_dir_all(target_dir.join("dir/aaa"))?;
        xattr::set(
            source_dir.join("dir/aaa"),
            "user.extract_package_test.foo",
            &[],
        )?;

        assert!(merge_directories(source_dir, target_dir).is_err());
        Ok(())
    }

    // Tests the case where the target directory have conflicting regular files.
    #[test]
    fn test_merge_directories_conflicting_regular_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let source_dir = &temp_dir.path().join("source");
        let target_dir = &temp_dir.path().join("target");

        std::fs::create_dir_all(source_dir.join("dir"))?;
        File::create(source_dir.join("dir/zzz"))?;
        std::fs::create_dir_all(target_dir.join("dir"))?;
        File::create(target_dir.join("dir/zzz"))?;

        assert!(merge_directories(source_dir, target_dir).is_err());
        Ok(())
    }

    // Tests the case where the target directory have conflicting files of different types.
    #[test]
    fn test_merge_directories_conflicting_typed_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let source_dir = &temp_dir.path().join("source");
        let target_dir = &temp_dir.path().join("target");

        std::fs::create_dir_all(source_dir.join("dir"))?;
        File::create(source_dir.join("dir/ppp"))?;
        std::fs::create_dir_all(target_dir.join("dir/ppp"))?;

        assert!(merge_directories(source_dir, target_dir).is_err());
        Ok(())
    }
}
