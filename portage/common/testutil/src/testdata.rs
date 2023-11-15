// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use std::path::Path;

/// Makes a copy of a directory, with certain files that aren't handled correctly by bazel renamed.
/// The following translations are performed:
/// * If you want a space in a file name, instead use "_SPACE_"
/// * If you want a "BUILD.bazel" file, instead use "BUILD.input.bazel"
/// * If you want a symlink foo pointing to bar, instead create a text file foo.symlink with the
///     content as "bar".
pub fn rename_bazel_input_testdata(dir: &Path) -> Result<fileutil::SafeTempDir> {
    let copy = fileutil::SafeTempDir::new()?;
    let dst_root = copy.path();

    // Everything in the runfiles is a symlink.
    for entry in walkdir::WalkDir::new(dir).follow_links(true).min_depth(1) {
        let entry = entry?;
        let src = entry.path();
        // Bazel complains about spaces in directory / filenames.
        let relative = src
            .strip_prefix(dir)?
            .to_str()
            .unwrap()
            .replace("_SPACE_", " ");

        let mut dst = dst_root.join(relative);
        let file_type = entry.file_type();
        // We add a custom file type for symlinks because everything in the runfiles is a symlink.
        if src.extension() == Some(std::ffi::OsStr::new("symlink")) {
            dst = dst.with_extension("");
            let target = std::fs::read_to_string(src)?;
            std::os::unix::fs::symlink(target.trim(), dst)?;
        } else if file_type.is_dir() {
            std::fs::create_dir(dst)?;
        } else if file_type.is_file() {
            // If the file is named BUILD.bazel, bazel treats it as a package.
            if src.file_name() == Some(std::ffi::OsStr::new("BUILD.input.bazel")) {
                dst = dst.with_file_name("BUILD.bazel");
            }
            std::fs::copy(src, dst)?;
        }
    }
    Ok(copy)
}
