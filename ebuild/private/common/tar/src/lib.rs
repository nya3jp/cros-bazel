// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use std::collections::HashSet;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use walkdir::WalkDir;

/// Creates a tar file at `dest` which contains all symlinks under `src`. Also removes all symlinks
/// under `src`.
pub fn move_symlinks_into_tar(src: &Path, dest: &Path) -> Result<()> {
    let file = std::fs::File::create(dest)?;
    let mut tar = tar::Builder::new(file);

    let mut written_parents = HashSet::new();

    for entry in WalkDir::new(&src).sort_by_file_name()
    // To make the output deterministic.
    {
        let entry = entry?;
        if !entry.file_type().is_symlink() {
            continue;
        }

        let link_source = entry.path().strip_prefix(src)?;
        let link_target = std::fs::read_link(&entry.path())?;

        // Write all parent directories if not written yet.
        // rev() to write parents before children.
        for parent in link_source
            .ancestors()
            .skip(1)
            .filter(|x| x != &Path::new("") && !written_parents.contains(*x))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            tar.append_dir(parent, src.join(parent))?;
            written_parents.insert(parent.to_path_buf());
        }

        // Write the symlink
        let mut header = tar::Header::new_gnu();
        let mode = std::fs::symlink_metadata(entry.path())?.mode();
        header.set_mode(mode);
        header.set_entry_type(tar::EntryType::Symlink);
        header.set_size(0);
        tar.append_link(&mut header, link_source, link_target)?;

        fileutil::remove_file_with_chmod(entry.path())?;
    }
    tar.finish()?;
    Ok(())
}
