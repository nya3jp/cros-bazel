// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use runfiles::Runfiles;
use std::path::Path;
use std::process::{Command, Stdio};

const BASE_DIR: &str = "cros/bazel/ebuild/private/cmd/sdk_from_archive";

fn base_test(input: &str, expected_status: i32) -> Result<()> {
    let r = Runfiles::create()?;

    let output_dir = tempfile::TempDir::new()?;

    let mut command = Command::new(r.rlocation(Path::new(BASE_DIR).join("sdk_from_archive")));

    let input = r.rlocation(Path::new(BASE_DIR).join(input));

    command
        .arg("--input")
        .arg(input)
        .arg("--output")
        .arg(output_dir.path());

    let status = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .status()?;

    assert_eq!(status.code(), Some(expected_status));

    if status.code() == Some(0) {
        let expected = output_dir.path().join("DURABLE_TREE");
        assert!(
            expected.try_exists()?,
            "Failed to find {} in {:?}",
            expected.display(),
            std::fs::read_dir(output_dir.path())?
                .map(|res| res.map(|e| e.path()))
                .collect::<Result<Vec<_>, std::io::Error>>()?
        );
    }

    Ok(())
}

#[test]
fn tar_xz_succeeds() -> Result<()> {
    base_test("archive.tar.xz", 0)
}

#[test]
fn tar_zst_succeeds() -> Result<()> {
    base_test("archive.tar.zst", 0)
}

#[test]
fn tar_fails() -> Result<()> {
    base_test("/NO/SUCH/FILE.tar.xz", 1)
}
