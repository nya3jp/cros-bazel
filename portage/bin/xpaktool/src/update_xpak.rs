// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{ensure, Context, Result};
use binarypackage::BinaryPackage;
use clap::Parser;
use std::path::PathBuf;

/// Parse a single key-value pair
fn parse_key_val(s: &str) -> Result<(String, String)> {
    let v: Vec<_> = s.split('=').collect();
    ensure!(v.len() == 2, "Invalid key-value: {:?}", s);
    Ok((v[0].to_string(), v[1].to_string()))
}

/// Compares two packages.
#[derive(Parser, Debug)]
pub struct UpdateXpakArgs {
    /// Portage binary package to update.
    #[arg(long)]
    binpkg: PathBuf,

    /// Values to override. Format: key=value.
    /// Note: value must be valid UTF-8.
    #[arg(value_parser = parse_key_val)]
    values: Vec<(String, String)>,
}

pub fn do_update_xpak(args: UpdateXpakArgs) -> Result<()> {
    let pkg = BinaryPackage::open(&args.binpkg).with_context(|| format!("{:?}", args.binpkg))?;

    let mut xpak = pkg.xpak().clone();

    for (k, v) in args.values {
        xpak.insert(k, v.into_bytes());
    }

    pkg.replace_xpak(&xpak)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdata::*;

    #[test]
    fn update_package() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        let src = &testdata(BINPKG)?;
        let dest = dir.join("out.tbz2");

        std::fs::copy(src, &dest)?;

        do_update_xpak(UpdateXpakArgs {
            binpkg: dest.clone(),
            values: vec![
                ("NEW_KEY".to_string(), "Hello World".to_string()),
                ("CHOST".to_string(), "x86_64-pc-linux-gnu\n".to_string()),
            ],
        })?;

        let src = BinaryPackage::open(src).with_context(|| format!("src: {src:?}"))?;
        let dest = BinaryPackage::open(&dest).with_context(|| format!("dest: {dest:?}"))?;

        assert_eq!(
            dest.xpak().get("NEW_KEY").unwrap(),
            "Hello World".as_bytes()
        );

        // Ensure key was replaced.
        assert_eq!(
            src.xpak().get("CHOST").unwrap(),
            "x86_64-cros-linux-gnu\n".as_bytes()
        );
        assert_eq!(
            dest.xpak().get("CHOST").unwrap(),
            "x86_64-pc-linux-gnu\n".as_bytes()
        );

        Ok(())
    }
}
