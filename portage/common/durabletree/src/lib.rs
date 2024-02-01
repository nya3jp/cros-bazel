// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod consts;
mod convert;
mod expand;
mod manifest;
#[cfg(test)]
mod tests;
mod util;

use crate::{convert::convert_impl, expand::expand_impl};
use anyhow::Result;
use consts::{MARKER_FILE_NAME, RAW_DIR_NAME};
use expand::ExtraDir;
use std::{
    fs::set_permissions,
    os::unix::prelude::PermissionsExt,
    path::{Path, PathBuf},
};
use tracing::instrument;
use util::list_user_xattrs;
use walkdir::WalkDir;

/// Works with *a durable tree*, a special directory format designed to preserve
/// file metadata in Bazel tree artifacts.
///
/// An arbitrary directory tree can be converted to a durable tree, and a
/// durable tree can be then converted to a set of directories that can be
/// mounted with overlayfs to reproduce the original directory. A durable tree
/// is safe as a Bazel tree artifact; that is, it does not contain non-regular
/// files (such as symlinks and character devices), and contains metadata
/// database to restore file metadata (permissions and user xattrs for now).
///
/// If a Bazel build action wants to output a directory to be mounted by later
/// actions, it can save the directory as a durable tree so that it is reliably
/// reproduced even if it is uploaded and downloaded from Bazel remote caches.
///
/// This type performs some cleanups on drop.
///
/// # Internal details
///
/// The following sections describe internal details of durable trees. They are
/// informational only and users must not rely on them as we might change them
/// for improvements.
///
/// ## Directory layout
///
/// A durable tree contains the following files:
///
/// - `DURABLE_TREE`: An empty marker file indicating that this directory is a
///   durable tree.
/// - `raw/...`: A directory containing regular files and directories.
/// - `manifest.json`: A JSON file that records original permissions and user
///   xattrs of files in the raw directory.
/// - `extra.tar.zst`: A zstd-compressed tarball containing special files that
///   cannot be part of Bazel tree artifacts, such as symlinks and character
///   device files.
///
/// These files are always created even if they're empty.
///
/// ## Restoration
///
/// Bazel drops some portion of tree artifacts when it uploads them to the
/// remote cache. We restore them when we detect such events.
///
/// Bazel forgets metadata (permissions and user xattrs) of files in the raw
/// directory when it uploads a durable tree to the remote cache. When we expand
/// a durable tree, we restore metadata according to the manifest JSON.
///
/// Bazel also forgets empty directories in the raw directory when it uploads a
/// durable tree to the remote cache. We can detect them by checking if a file
/// path in the manifest actually exists in the raw directory; if it's missing,
/// it is an empty directory removed by Bazel, so we recreate it.
///
/// Since restoration is a heavy task when the tree contains thousands of files,
/// we record in the top directory's xattrs whether we have already restored and
/// skip it when it's set. We also use flock(2) on the top directory to prevent
/// multiple restore operations from running in parallel.
///
/// ## Hot state
///
/// When a program creates a durable tree with [`DurableTree::convert`], it is
/// initially marked as *hot*. A durable tree is considered hot if its root
/// directory has the permission 0700.
///
/// It is an error to attempt to expand a hot durable tree with
/// [`DurableTree::expand`] because it would mark the tree "already restored"
/// (as we explained in the previous section), while Bazel would modify
/// permissions of files in the tree after the current Bazel action finishes,
/// which results in corrupted permissions.
///
/// Therefore, a Bazel action should convert its output directory to a durable
/// tree at the end of its execution and should never attempt to expand it.
/// Once the action finishes, Bazel modifies permissions of the durable tree to
/// 0555, which essentially removes the hot state. From this point, Bazel won't
/// modify file permissions anymore, so it is safe to expand the durable tree
/// and set the "already restored" marker in other actions.
///
/// If you need to expand a hot durable tree in unit tests, you can use
/// [`DurableTree::cool_down_for_testing`] to simulate the "cool down" process
/// of Bazel.
///
/// ## Limitations
///
/// We don't record xattrs in the extra tarball because the tar library we're
/// using now doesn't support the PAX format needed to record xattrs:
/// https://github.com/alexcrichton/tar-rs/issues/102
///
/// This is fine for now because the extra tarball only contains special files
/// that we can't set xattrs to (besides ancestor directories of special files;
/// see the next section).
///
/// ## Layer ordering
///
/// The same directory might be recorded in both the raw directory and the extra
/// tarball. Due to the limitations mentioned above, directories in the extra
/// archive might be missing some metadata. Therefore the raw directory must
/// take precedence over the extra tarball.
pub struct DurableTree {
    _extra_dir: ExtraDir,

    // A list of layer directories to return from `layers`. This is a subset of
    // the raw directory and the extra directory.
    layer_dirs: Vec<PathBuf>,
}

impl DurableTree {
    /// Checks if a specified directory is a durable tree.
    pub fn try_exists(root_dir: &Path) -> Result<bool> {
        let metadata = root_dir.metadata()?;
        if !metadata.is_dir() {
            return Ok(false);
        }
        Ok(root_dir.join(MARKER_FILE_NAME).try_exists()?)
    }

    /// Converts a plain directory to a durable tree in place.
    ///
    /// It is an error to attempt to convert a directory that is already a
    /// durable tree.
    #[instrument]
    pub fn convert(root_dir: &Path) -> Result<()> {
        convert_impl(root_dir)
    }

    /// Expands a durable tree.
    ///
    /// This function mounts tmpfs on a generated extra directory, which
    /// requires the calling process to have privilege to mount tmpfs.
    ///
    /// Once it succeeds, you can call [`DurableTree::layers`] to get a list of
    /// directories to mount with overlayfs.
    ///
    /// Expanding a durable tree may modify/create some files/directories in the
    /// directory to restore some data forgotten by Bazel on saving the tree
    /// artifact to the remote cache. But it is safe to expand the same durable
    /// tree from multiple threads and processes in parallel.
    #[instrument]
    pub fn expand(root_dir: &Path) -> Result<Self> {
        let raw_dir = root_dir.join(RAW_DIR_NAME);
        let extra_dir = expand_impl(root_dir)?;

        // Compute a list of layer directories to return from `layers`. This is
        // a subset of `raw_dir` and `extra_dir`, but we might be able to skip
        // some of them when they're empty.
        // See the comment of `DurableTree` for the reason the raw directory
        // takes precedence.
        let mut layer_dirs = Vec::new();
        if dir_has_child(extra_dir.path())? {
            layer_dirs.push(extra_dir.path().to_path_buf());
            // We can omit the raw directory only when the extra directory is
            // omitted.
            layer_dirs.push(raw_dir.clone());
        } else if dir_has_child(&raw_dir)? {
            layer_dirs.push(raw_dir.clone());
        }

        Ok(DurableTree {
            _extra_dir: extra_dir,
            layer_dirs,
        })
    }

    /// Returns a list of directories to mount with overlayfs to reproduce the
    /// original directory.
    ///
    /// Directories are listed in the mount order. That is, a former directory
    /// is overridden by a latter directory.
    ///
    /// Depending on the content of a durable tree, this method may return the
    /// different number of directories. In the case of an empty tree, this
    /// method may return an empty vector.
    pub fn layers(&self) -> Vec<&Path> {
        self.layer_dirs.iter().map(|dir| dir.as_path()).collect()
    }

    /// Resets the "hot" state of a durable tree by resetting permissions of
    /// files under a directory just like Bazel does.
    ///
    /// Use this function only in tests if you want to call
    /// [`DurableTree::expand`] on a durable tree created with
    /// [`DurableTree::convert`].
    pub fn cool_down_for_testing(root_dir: &Path) -> Result<()> {
        for entry in WalkDir::new(root_dir) {
            let entry = entry?;
            let path = entry.path();

            // Make the file writable to allow listing/clearing user xattrs.
            set_permissions(path, PermissionsExt::from_mode(0o755))?;

            // Clear all user xattrs.
            for key in list_user_xattrs(path)? {
                xattr::remove(path, key)?;
            }

            // Set the file permission to 0555 that Bazel typically uses.
            set_permissions(path, PermissionsExt::from_mode(0o555))?;
        }
        Ok(())
    }
}

fn dir_has_child(dir: &Path) -> Result<bool> {
    Ok(dir.read_dir()?.next().is_some())
}
