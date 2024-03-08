// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{path::Path, process::Command};

use anyhow::{ensure, Result};
use runfiles::Runfiles;
use tempfile::tempdir;
use testutil::compare_with_golden_data;

const TESTDATA_DIR: &str = "bazel/portage/bin/alchemist/src/bin/alchemist/testdata";

#[test]
fn test_generate_repo() -> Result<()> {
    let temp_dir = tempdir()?;
    let temp_dir = temp_dir.path();
    let output_dir = temp_dir.join("portage-repo");
    let deps_file = temp_dir.join("output_repos.json");

    let input_dir = testutil::rename_bazel_input_testdata(
        &testutil::runfiles_root()?.join(TESTDATA_DIR).join("input"),
    )?;
    let input_dir = input_dir.path();

    let runfiles = Runfiles::create()?;
    let alchemist_path =
        runfiles.rlocation("cros/bazel/portage/bin/alchemist/src/bin/alchemist/alchemist");

    let status = Command::new(alchemist_path)
        .args([
            "--board=amd64-generic",
            "--use-portage-site-configs=false",
            "--force-accept-9999-ebuilds=false",
            &format!("--source-dir={}", input_dir.display()),
            "generate-repo",
            &format!("--output-dir={}", output_dir.display()),
            &format!("--output-repos-json={}", deps_file.display()),
        ])
        .status()?;
    ensure!(status.success());

    // trace.json changes every time we run.
    std::fs::remove_file(output_dir.join("trace.json"))?;

    compare_with_golden_data(&output_dir, &Path::new(TESTDATA_DIR).join("golden"))?;

    Ok(())
}
