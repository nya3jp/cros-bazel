// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};

use anyhow::Result;

/// Writes files to a directory.
///
/// `base_dir` is the base directory to write files under. `files` is an
/// iterator of `(path, content)` pairs where `path` is a relative path from
/// `base_dir` and `content` is the content of the file to be written.
pub fn write_files<'a, P: AsRef<Path> + 'a, D: AsRef<str> + 'a, I: IntoIterator<Item = (P, D)>>(
    base_dir: impl AsRef<Path>,
    files: I,
) -> Result<()> {
    let dir = base_dir.as_ref();

    for (rel_path, content) in files.into_iter() {
        let path = dir.join(rel_path.as_ref());
        let content = content.as_ref();

        create_dir_all(path.parent().unwrap())?;

        let mut file = File::create(&path)?;
        file.write_all(content.as_bytes())?;
    }

    Ok(())
}
