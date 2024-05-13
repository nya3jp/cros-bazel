// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use runfiles::Runfiles;
use std::fs::{create_dir_all, set_permissions};
use std::fs::{File, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use testutil::fakefs_chown;

const BASE_DIR: &str = "cros/bazel/portage/bin/sdk_to_archive";

fn list_tar(path: &Path) -> Result<String> {
    let mut cmd = Command::new("tar");
    cmd.arg("-tvv").arg("-f").arg(path);

    let output = cmd.output()?;

    assert!(
        output.status.success(),
        "Command {:?} failed with {:?}",
        cmd,
        output
    );

    Ok(String::from_utf8(output.stdout)?)
}

fn create_sysroot(root: &Path) -> Result<()> {
    for (dir, perms) in [
        ("", 0o755),
        ("./etc", 0o750),
        ("./home/bob", 0o750),
        ("./tmp", 0o1777),
    ] {
        let absolute = root.join(dir);
        create_dir_all(&absolute).with_context(|| format!("mkdir -p {absolute:?}"))?;
        set_permissions(&absolute, Permissions::from_mode(perms))?;
    }

    File::create(root.join("./etc/init.conf"))?;
    File::create(root.join("./home/bob/.bashrc"))?;

    fakefs_chown("1000:2000", &root.join("home/bob"))?;
    fakefs_chown("1000:2000", &root.join("home/bob/.bashrc"))?;

    Ok(())
}

#[test]
fn tar_round_trip() -> Result<()> {
    let r = Runfiles::create()?;

    let tmp_dir = PathBuf::from(std::env::var("TEST_TMPDIR").context("TEST_TMPDIR is not set")?);

    let output = tmp_dir.join("actual.tar.zst");

    let input = tmp_dir.join("root");
    create_sysroot(&input)?;

    let status = Command::new(r.rlocation(Path::new(BASE_DIR).join("sdk_to_archive")))
        .env("RUST_BACKTRACE", "full")
        .arg("--layer")
        .arg(&input)
        .arg("--output")
        .arg(&output)
        .status()?;

    assert!(status.success());

    let listing = list_tar(&output)?;

    assert_eq!(
        r#"drwxr-xr-x 0/0               0 1970-01-01 00:00 ./
drwxr-x--- 0/0               0 1970-01-01 00:00 ./etc/
-rw-r--r-- 0/0               0 1970-01-01 00:00 ./etc/init.conf
drwxr-xr-x 0/0               0 1970-01-01 00:00 ./home/
drwxr-x--- 1000/2000         0 1970-01-01 00:00 ./home/bob/
-rw-r--r-- 1000/2000         0 1970-01-01 00:00 ./home/bob/.bashrc
drwxrwxrwt 0/0               0 1970-01-01 00:00 ./tmp/
"#,
        listing
    );

    Ok(())
}
