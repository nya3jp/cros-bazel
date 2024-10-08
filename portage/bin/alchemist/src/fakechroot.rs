// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::common::CHROOT_SOURCE_DIR;
use crate::config::makeconf::generate::generate_make_conf_for_board;
use crate::fileops::execute_file_ops;
use crate::fileops::FileOps;
use crate::repository::Repository;
use crate::repository::RepositoryLookup;
use crate::toolchain::load_toolchains;
use crate::toolchain::ToolchainConfig;
use std::fs::create_dir_all;
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

const OLD_ROOT_NAME: &str = ".old-root";

/// Provides a way to translate paths inner and outer paths.
/// This is useful when running inside a container and you have a "bind mount"
/// between the host and container.
#[derive(Clone, Debug)]
pub struct PathTranslator {
    inner: PathBuf,
    outer: PathBuf,
}

impl PathTranslator {
    fn new(inner: impl AsRef<Path>, outer: impl AsRef<Path>) -> Self {
        Self {
            inner: inner.as_ref().to_owned(),
            outer: outer.as_ref().to_owned(),
        }
    }

    /// Creates a PathTranslator which does nothing.
    pub fn noop() -> Self {
        Self::new(Path::new(""), Path::new(""))
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

/// Enters new user/mount namespace to prepare for privileged filesystem
/// operations such as mount(2) and pivot_root(2).
fn enter_namespaces() -> Result<()> {
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
    Ok(())
}

/// Hides the contents of specified directories.
///
/// After successfully calling this function, specified directories should be
/// empty and writable. Files under those directories are not deleted.
///
/// You need to call [`enter_namespaces`] in advance.
fn hide_directories(dirs_to_hide: &[&Path]) -> Result<()> {
    let root_dir = Path::new("/");

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

    // Create directories to hide.
    for dir_to_hide in dirs_to_hide.iter() {
        let new_hide_dir = new_root_dir.join(dir_to_hide.strip_prefix("/")?);
        create_dir_all(new_hide_dir)?;
    }

    // Create symlinks to files in the original filesystem.
    // The old root filesystem will be mounted at `/.old-root`. We will create
    // symlinks to files that exist in the the original filesystem but not in
    // `new_root_dir`, except those directories to hide.
    for new_dir_entry in WalkDir::new(new_root_dir) {
        // Iterate on all directories under [new_root_dir].
        // Example: ${new_root_dir}/mnt
        let new_dir_entry = new_dir_entry?;
        if !new_dir_entry.file_type().is_dir() {
            continue;
        }

        let rel_path = new_dir_entry.path().strip_prefix(new_root_dir)?;
        let orig_dir = root_dir.join(rel_path);

        // Don't process directories to hide.
        if dirs_to_hide
            .iter()
            .any(|dir_to_hide| *dir_to_hide == orig_dir)
        {
            continue;
        }

        // Enumerate files in the corresponding directory in the original
        // filesystem.
        // Example: /mnt
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
        // Example: ${new_root_dir}/mnt/disk -> /.old-root/mnt/disk
        for orig_dir_entry in orig_dir_entries {
            let orig_dir_entry = orig_dir_entry?;
            let source = new_dir_entry.path().join(orig_dir_entry.file_name());
            let target = root_dir
                .join(OLD_ROOT_NAME)
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
    create_dir(new_root_dir.join(OLD_ROOT_NAME))?;

    // Finally, call pivot_root(2).
    pivot_root(new_root_dir, &new_root_dir.join(OLD_ROOT_NAME))?;

    Ok(())
}

pub fn host_config_file_ops(profile: Option<&Path>) -> Vec<FileOps> {
    vec![
        FileOps::symlink(
            "/etc/make.conf",
            "/mnt/host/source/src/third_party/chromiumos-overlay/chromeos/config/make.conf.amd64-host",
        ),
        FileOps::plainfile(
            "/etc/make.conf.board_setup",
            r#"
# Created by cros_sysroot_utils from --board=amd64-host.
ARCH="amd64"
BOARD_OVERLAY="/mnt/host/source/src/overlays/overlay-amd64-host"
BOARD_USE="amd64-host"
CHOST="x86_64-pc-linux-gnu"
PORTDIR_OVERLAY="/mnt/host/source/src/overlays/overlay-amd64-host"
"#,
        ),
        FileOps::plainfile("/etc/make.conf.host_setup", ""),
        FileOps::plainfile("/etc/make.conf.user", ""),
        FileOps::symlink(
            "/etc/portage/make.profile",
            profile.unwrap_or(
                Path::new("/mnt/host/source/src/third_party/chromiumos-overlay/profiles/default/linux/amd64/10.0/sdk")),
        ),
    ]
}

/// Generates the portage config for the host SDK.
///
/// Instead of depending on an extracted SDK tarball, we hard code the config
/// here. The host config is relatively simple, so it shouldn't be changing
/// that often.
fn generate_host_configs() -> Result<()> {
    let mut ops = vec![
        // Host specific files
        FileOps::symlink(
            "/etc/ld.so.cache",
            Path::new("/").join(OLD_ROOT_NAME).join("etc/ld.so.cache"),
        ),
        FileOps::symlink(
            "/etc/ld.so.conf",
            Path::new("/").join(OLD_ROOT_NAME).join("etc/ld.so.conf"),
        ),
        FileOps::symlink(
            "/etc/ld.so.conf.d",
            Path::new("/").join(OLD_ROOT_NAME).join("etc/ld.so.conf.d"),
        ),
    ];

    ops.extend(host_config_file_ops(None));

    execute_file_ops(&ops, Path::new("/"))
}

/// Generates the portage configuration for the board.
pub fn target_config_file_ops(
    board: &str,
    profile_path: &Path,
    repos: &[&Repository],
    toolchains: &ToolchainConfig,
    include_provided: bool,
) -> Result<Vec<FileOps>> {
    let mut files = vec![
        FileOps::symlink (
            "/etc/make.conf",
            "/mnt/host/source/src/third_party/chromiumos-overlay/chromeos/config/make.conf.generic-target",
        ),
        FileOps::symlink (
            "/etc/make.conf.user",
            "/etc/make.conf.user",
        ),
        FileOps::symlink(
            "/etc/portage/make.profile",
            profile_path,
        ),
    ];

    if include_provided {
        files.push(
            // TODO(b/266979761): Remove the need for this list
            FileOps::plainfile(
                "/etc/portage/profile/package.provided",
                r#"
sys-devel/gcc-10.2.0-r30
sys-libs/glibc-9999
dev-lang/go-1.20.2-r2
"#,
            ),
        );
    }

    files.extend(generate_make_conf_for_board(board, repos, toolchains)?);

    Ok(files)
}

fn generate_board_configs(
    board: &str,
    profile_path: &Path,
    repos: &[&Repository],
    toolchains: &ToolchainConfig,
) -> Result<()> {
    let board_root = Path::new("/build").join(board);

    execute_file_ops(
        &target_config_file_ops(board, profile_path, repos, toolchains, true)?,
        &board_root,
    )?;

    Ok(())
}

/// Generates the portage configuration for the board amd64-host board.
/// It has a couple differences from the chromeos target board:
/// 1) No need to generate a package.provided since we want the compilers
/// 2) The make.conf target is different.
/// 3) We need to generate a make.conf.host_setup instead of a make.conf.board.
pub fn target_host_config_file_ops(
    board: &str,
    profile_path: &Path,
    repos: &[&Repository],
    toolchains: &ToolchainConfig,
) -> Result<Vec<FileOps>> {
    let mut files = vec![
        FileOps::symlink (
            "/etc/make.conf",
            "/mnt/host/source/src/third_party/chromiumos-overlay/chromeos/config/make.conf.amd64-host",
        ),
        FileOps::symlink (
            "/etc/make.conf.user",
            "/etc/make.conf.user",
        ),
        FileOps::symlink(
            "/etc/portage/make.profile",
            profile_path,
        ),
    ];

    files.extend(generate_make_conf_for_board(board, repos, toolchains)?);

    Ok(files)
}

fn generate_sdk_board_configs(
    board: &str,
    profile_path: &Path,
    repos: &[&Repository],
    toolchains: &ToolchainConfig,
) -> Result<()> {
    let board_root = Path::new("/build").join(board);

    execute_file_ops(
        &target_host_config_file_ops(board, profile_path, repos, toolchains)?,
        &board_root,
    )?;

    Ok(())
}

// Generates a /build/$BOARD directory for each target that contains the portage
// config required to build packages.
fn generate_target_configs(targets: &[&BoardTarget]) -> Result<()> {
    // We throw away the repos and toolchain after we generate the files
    // because we need to evaluate the PORTDIR and PORTDIR_OVERLAY variables
    // as they are defined in the make.conf files.
    let lookup = RepositoryLookup::new(
        Path::new("/mnt/host/source"),
        vec!["src/private-overlays", "src/overlays", "src/third_party"],
    )?;

    for target in targets {
        let repos = lookup.create_repository_set(target.board)?;

        let profile_path = repos
            .primary()
            .base_dir()
            .join("profiles")
            .join(target.profile);

        let repos: Vec<_> = repos.get_partially_ordered_repos().iter().collect();

        // TODO(b/314189420): This method iterates the repos in reverse. If the
        // "partner" or "internal" repos are at the end of the list, their
        // toolchain.conf will be chosen as the primary. Thankfully we don't
        // have any toolchain.conf in these repos. Ideally we just drop support
        // for toolchain.conf all together and instead specify a CHOST in all
        // the board profiles.
        let toolchains = load_toolchains(&repos)?;

        if target.board == "amd64-host" {
            generate_sdk_board_configs(target.board, &profile_path, &repos, &toolchains)?;
        } else {
            generate_board_configs(target.board, &profile_path, &repos, &toolchains)?;
        }
    }

    Ok(())
}

pub struct BoardTarget<'a> {
    pub board: &'a str,
    pub profile: &'a str,
}

/// Enters a fake CrOS chroot.
///
/// A fake CrOS chroot is not a real CrOS chroot, but it's more like a unified
/// view of a simulated CrOS chroot with minimal configuration files and the
/// original system environment. Specifically, a fake CrOS chroot provides
/// /mnt/host/source, /build, /etc/portage and several other files in the CrOS
/// chroot needed to evaluate Portage profiles and ebuilds. The process can
/// still access other file paths on the system, e.g. Bazel runfiles.
///
/// This function requires the current process to be single-threaded for
/// unshare(2) calls to succeed. Make sure to call this function early in your
/// program, before starting threads.
///
/// # Arguments
///
/// * `targets` - The board and profile pairs to generate /build/$BOARD
///               ROOTs for.
/// * `source_dir` - The `repo` root directory. i.e., directory that contains
///   the `.repo` directory. This will be mounted at /mnt/host/source.
///
/// It returns [`PathTranslator`] that can be used to translate file paths in
/// the fake chroot to the original paths.
pub fn enter_fake_chroot(targets: &[&BoardTarget], source_dir: &Path) -> Result<PathTranslator> {
    // Canonicalize `source_dir` so it can be used in symlink targets.
    // Do this before entering the namespace to avoid including "/.old-root" in
    // the resolved path.
    let source_dir = source_dir.canonicalize()?;

    enter_namespaces()?;

    let source_mount_point = Path::new("/mnt/host/source");
    let inside_cros_chroot = source_mount_point.try_exists()?;
    let mut dirs_to_hide = vec![Path::new("/build"), Path::new("/etc")];
    if !inside_cros_chroot {
        dirs_to_hide.push(Path::new("/mnt/host"));
    }

    hide_directories(&dirs_to_hide)?;

    // Create /mnt/host/source symlink.
    if !inside_cros_chroot {
        symlink(&source_dir, source_mount_point)?;
    }

    let translator = PathTranslator::new(CHROOT_SOURCE_DIR, &source_dir);

    // Generate configs.
    generate_host_configs()?;

    generate_target_configs(targets)?;

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
