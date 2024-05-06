// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, ensure, Result};
use runfiles::Runfiles;

use std::path::{Path, PathBuf};
use std::process::Command;

pub fn fakefs_path() -> Result<PathBuf> {
    let r = Runfiles::create()?;

    let fakefs = runfiles::rlocation!(r, "cros/bazel/portage/bin/fakefs/fakefs_/fakefs");
    if !fakefs.try_exists()? {
        bail!("{} doesn't exist", fakefs.display());
    }

    Ok(fakefs)
}

/// Uses fakefs to chown the specified file.
pub fn fakefs_chown(user_group: &str, path: &Path) -> Result<()> {
    let mut command = Command::new(fakefs_path()?);
    command.arg("chown").arg(user_group).arg(path);

    let status = command.status()?;

    ensure!(status.success(), "{:?} failed.", command);

    Ok(())
}

pub struct Metadata {
    uid: u32,
    gid: u32,
    mode: u32,
}

impl Metadata {
    fn new(uid: u32, gid: u32, mode: u32) -> Self {
        Self { uid, gid, mode }
    }

    pub fn gid(&self) -> u32 {
        self.gid
    }

    pub fn uid(&self) -> u32 {
        self.uid
    }

    pub fn mode(&self) -> u32 {
        self.mode
    }

    pub fn permissions(&self) -> u32 {
        self.mode & 0o7777
    }
}

/// Calls `stat` using fakefs on the specified file.
pub fn fakefs_stat(path: &Path) -> Result<Metadata> {
    let mut command = Command::new(fakefs_path()?);
    command
        .arg("stat")
        .arg("--format")
        .arg("%u %g %f")
        .arg(path);
    let output = command.output()?;

    ensure!(
        output.status.success(),
        "{:?} failed with {}",
        command,
        output.status
    );

    let output = String::from_utf8(output.stdout)?;

    let mut data = output.trim().split_ascii_whitespace();
    let uid = data.next().unwrap().parse()?;
    let gid = data.next().unwrap().parse()?;
    let mode = u32::from_str_radix(data.next().unwrap(), 16)?;

    Ok(Metadata::new(uid, gid, mode))
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use std::fs::set_permissions;
    use std::fs::File;
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[test]
    fn test_fake_chown() -> Result<()> {
        let tmp_dir =
            PathBuf::from(std::env::var("TEST_TMPDIR").context("TEST_TMPDIR is not set")?);

        let file_path = tmp_dir.join("foo");
        File::create(&file_path)?;
        set_permissions(&file_path, Permissions::from_mode(0o640))?;

        fakefs_chown("100:200", &file_path)?;

        let stat = fakefs_stat(&file_path)?;

        assert_eq!(stat.uid(), 100);
        assert_eq!(stat.gid(), 200);
        assert_eq!(stat.permissions(), 0o640);

        Ok(())
    }
}
