// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
use anyhow::{bail, Result};
use path_absolutize::Absolutize;
use std::{path::Path, path::PathBuf, str::FromStr};

fn from_str(spec: &str) -> Result<(PathBuf, PathBuf)> {
    let (first, second) = cliutil::split_key_value(spec)?;
    Ok((
        PathBuf::from(if first == "/" {
            "/"
        } else {
            first.trim_end_matches('/')
        }),
        PathBuf::from(second),
    ))
}

#[derive(Debug, Clone)]
pub struct OverlayInfo {
    pub mount_dir: PathBuf,
    pub image_path: PathBuf,
}

impl FromStr for OverlayInfo {
    type Err = anyhow::Error;
    fn from_str(spec: &str) -> Result<Self> {
        let (mount_dir, image_path) = from_str(spec)?;
        Ok(Self {
            mount_dir,
            image_path,
        })
    }
}

#[derive(Debug, Clone)]
pub struct BindMount {
    pub mount_path: PathBuf,
    pub source: PathBuf,
}

impl FromStr for BindMount {
    type Err = anyhow::Error;
    fn from_str(spec: &str) -> Result<Self> {
        let (mount_path, source) = from_str(spec)?;
        Ok(Self { mount_path, source })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlayType {
    Dir,
    Squashfs,
    Tar,
}

impl OverlayType {
    pub fn detect<P: AsRef<Path>>(image_path: P) -> Result<Self> {
        let image_path = image_path.as_ref().absolutize()?;

        let file_name = image_path
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or_default();
        let extension = image_path
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or_default();
        if std::fs::metadata(&image_path)?.is_dir() {
            Ok(OverlayType::Dir)
        } else if extension == "squashfs" {
            Ok(OverlayType::Squashfs)
        } else if file_name.ends_with(".tar.zst") || file_name.ends_with(".tar") {
            Ok(OverlayType::Tar)
        } else {
            bail!("unsupported file type: {:?}", image_path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runfiles::Runfiles;

    #[test]
    fn detect_overlay_type_works() -> Result<()> {
        let r = Runfiles::create()?;
        let testdata = PathBuf::from("cros/bazel/ebuild/private/common/makechroot/testdata/");
        assert_eq!(
            OverlayType::detect(r.rlocation(testdata.join("example.squashfs")))?,
            OverlayType::Squashfs
        );
        assert_eq!(
            OverlayType::detect(r.rlocation(testdata.join("example.tar.zst")))?,
            OverlayType::Tar
        );
        assert_eq!(
            OverlayType::detect(r.rlocation(testdata.join("example.tar")))?,
            OverlayType::Tar
        );
        assert_eq!(OverlayType::detect(Path::new("/dev"))?, OverlayType::Dir);
        assert!(OverlayType::detect(Path::new("/dev/null")).is_err());

        Ok(())
    }
}
