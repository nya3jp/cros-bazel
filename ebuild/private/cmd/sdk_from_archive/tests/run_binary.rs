// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use std::process::{Command, Stdio};

fn base_test(input: &str, expected_status: i32) -> Result<()> {
    let output_dir = tempfile::TempDir::new()?;
    let symlink_file = tempfile::NamedTempFile::new()?;

    let mut command = Command::new(env!("CARGO_BIN_EXE_sdk_from_archive"));

    command
        .arg("--input")
        .arg(input)
        .arg("--output-dir")
        .arg(output_dir.path())
        .arg("--output-symlink-tar")
        .arg(symlink_file.path());

    let status = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .status()?;

    assert_eq!(status.code(), Some(expected_status));

    Ok(())
}

#[test]
fn tar_succeeds() -> Result<()> {
    base_test(concat!(env!("CARGO_MANIFEST_DIR"), "/archive.tar.xz"), 0)
}

#[test]
fn tar_fails() -> Result<()> {
    base_test("/NO/SUCH/FILE.tar.xz", 2)
}
