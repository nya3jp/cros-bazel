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
use std::path::{Path, PathBuf};
use tracing::instrument;

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
    raw_dir: PathBuf,
    extra_dir: ExtraDir,
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
        let extra_dir = expand_impl(root_dir)?;

        Ok(DurableTree {
            raw_dir: root_dir.join(RAW_DIR_NAME),
            extra_dir,
        })
    }

    /// Returns a list of directories to mount with overlayfs to reproduce the
    /// original directory.
    ///
    /// Directories are listed in the mount order. That is, a former directory
    /// is overridden by a latter directory.
    pub fn layers(&self) -> Vec<&Path> {
        // See the comment of `DurableTree` for the reason the raw directory
        // takes precedence.
        vec![self.extra_dir.path(), &self.raw_dir]
    }

    /// Similar to [`DurableTree::layers`], but consumes [`DurableTree`].
    ///
    /// After calling this function, it is your responsibility to clean layers
    /// up. Since layers might involve mounting file systems, properly cleaning
    /// them up by yourself is difficult. Use this function only as a last
    /// resort when you make changes to the file system mounts that prevents the
    /// default clean-up from working properly, e.g. calling pivot_root(2).
    ///
    /// If you really have to clean things up by yourself, this is a way that
    /// will work:
    /// - Enter a new mount namespace before calling [`DurableTree::expand`].
    /// - After you're done with durable trees, remove the whole $TMPDIR from
    ///   a process outside of the mount namespace.
    #[must_use]
    pub fn into_layers(self) -> Vec<PathBuf> {
        vec![self.extra_dir.into_path(), self.raw_dir]
    }
}
