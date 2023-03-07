// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Result};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct XpakSpec {
    pub xpak_header: String,
    pub optional: bool,
    pub target_path: PathBuf,
}

impl FromStr for XpakSpec {
    type Err = anyhow::Error;
    // Spec format: <XPAK key>=[?]<outside path>
    // If =? is used, an empty file is written if the key doesn't exist
    fn from_str(spec: &str) -> Result<Self> {
        let (xpak_header, target_path) = cliutil::split_key_value(spec)?;
        let (target_path, optional) = if let Some(target_path) = target_path.strip_prefix('?') {
            (target_path, true)
        } else {
            (target_path, false)
        };
        Ok(Self {
            xpak_header: xpak_header.to_string(),
            optional,
            target_path: PathBuf::from_str(target_path)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct OutputFileSpec {
    pub inside_path: String,
    pub target_path: PathBuf,
}

impl FromStr for OutputFileSpec {
    type Err = anyhow::Error;
    // Spec format: <inside path>=<outside path>
    fn from_str(spec: &str) -> Result<Self> {
        let (inside_path, target_path) = cliutil::split_key_value(spec)?;
        let res = Self {
            inside_path: inside_path.to_string(),
            target_path: PathBuf::from_str(target_path)?,
        };
        if !res.inside_path.starts_with('/') {
            bail!(
                "Invalid overlay spec: {spec}, {0:?} must be absolute",
                res.inside_path
            );
        }
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_xpak_file_required() -> Result<()> {
        let spec = XpakSpec::from_str("a=b")?;
        assert_eq!(spec.xpak_header, "a");
        assert_eq!(spec.target_path, PathBuf::from("b"));
        assert!(!spec.optional);

        Ok(())
    }

    #[test]
    fn parse_xpak_file_optional() -> Result<()> {
        let spec = XpakSpec::from_str("a=?b")?;
        assert_eq!(spec.xpak_header, "a");
        assert_eq!(spec.target_path, PathBuf::from("b"));
        assert!(spec.optional);

        Ok(())
    }
}
