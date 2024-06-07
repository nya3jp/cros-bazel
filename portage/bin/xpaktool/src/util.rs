// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use std::fs::File;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

pub fn files_size_equal(path_a: &Path, path_b: &Path) -> Result<bool> {
    let metadata_a = std::fs::metadata(path_a).with_context(|| format!("{path_a:?}"))?;
    let metadata_b = std::fs::metadata(path_b).with_context(|| format!("{path_b:?}"))?;

    Ok(metadata_a.size() == metadata_b.size())
}

pub fn files_contents_equal(path_a: &Path, path_b: &Path) -> Result<bool> {
    let mut file_a = File::open(path_a).with_context(|| format!("{path_a:?}"))?;
    let mut file_b = File::open(path_b).with_context(|| format!("{path_b:?}"))?;

    reader_contents_equal(&mut file_a, &mut file_b)
}

pub fn reader_contents_equal(reader_a: &mut impl Read, reader_b: &mut impl Read) -> Result<bool> {
    let mut buffer_a = [0u8; 4096];
    let mut buffer_b = [0u8; 4096];

    loop {
        let n_a = reader_a.read(&mut buffer_a)?;
        let n_b = reader_b.read(&mut buffer_b)?;

        if buffer_a[..n_a] != buffer_b[..n_b] {
            return Ok(false);
        }

        if n_a == 0 {
            return Ok(true);
        }
    }
}

pub fn files_equal(left: &Path, right: &Path) -> Result<bool> {
    Ok(files_size_equal(left, right)? && files_contents_equal(left, right)?)
}
