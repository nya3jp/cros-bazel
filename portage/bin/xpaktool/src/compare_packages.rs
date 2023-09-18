// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use binarypackage::BinaryPackage;
use clap::Parser;
use itertools::Itertools;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::path::PathBuf;

/// Compares two packages.
#[derive(Parser, Debug)]
pub struct ComparePackagesArgs {
    #[arg(name = "PACKAGE-A", help = "Portage binary packages to compare")]
    package_a: PathBuf,
    #[arg(name = "PACKAGE-B", help = "Portage binary packages to compare")]
    package_b: PathBuf,
}

fn files_size_equal(path_a: &Path, path_b: &Path) -> Result<bool> {
    let metadata_a = std::fs::metadata(path_a).with_context(|| format!("{path_a:?}"))?;
    let metadata_b = std::fs::metadata(path_b).with_context(|| format!("{path_b:?}"))?;

    Ok(metadata_a.size() == metadata_b.size())
}

fn files_contents_equal(path_a: &Path, path_b: &Path) -> Result<bool> {
    let mut file_a = File::open(path_a).with_context(|| format!("{path_a:?}"))?;
    let mut file_b = File::open(path_b).with_context(|| format!("{path_b:?}"))?;

    reader_contents_equal(&mut file_a, &mut file_b)
}

fn reader_contents_equal(reader_a: &mut impl Read, reader_b: &mut impl Read) -> Result<bool> {
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

fn packages_equal(path_a: &Path, path_b: &Path) -> Result<bool> {
    if files_size_equal(path_a, path_b)? {
        println!("File size equal - ✅");
        if files_contents_equal(path_a, path_b)? {
            println!("File contents equal - ✅");
            return Ok(true);
        } else {
            println!("File contents equal - ❌");
        }
    } else {
        println!("File sizes equal - ❌");
    }

    let mut pkg_a = BinaryPackage::open(path_a).with_context(|| format!("{path_a:?}"))?;
    let mut pkg_b = BinaryPackage::open(path_b).with_context(|| format!("{path_b:?}"))?;

    if reader_contents_equal(
        &mut pkg_a.new_tarball_reader()?,
        &mut pkg_b.new_tarball_reader()?,
    )? {
        println!("Tarball contents equal - ✅");
    } else {
        println!("Tarball contents equal - ❌");
        // TODO: compare tarball contents
    }

    if reader_contents_equal(&mut pkg_a.new_xpak_reader()?, &mut pkg_b.new_xpak_reader()?)? {
        println!("XPAK contents equal - ✅");
    } else {
        println!("XPAK contents equal - ❌");
        // TODO: compare xpak contents
    }

    Ok(false)
}
pub fn do_compare_packages(args: ComparePackagesArgs) -> Result<()> {
    if packages_equal(&args.package_a, &args.package_b)? {
        Ok(())
    } else {
        bail!("Packages are not equal")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdata::*;

    #[test]
    fn packages_equal_match() -> Result<()> {
        assert!(packages_equal(&testdata(BINPKG)?, &testdata(BINPKG)?)?);

        Ok(())
    }

    #[test]
    fn packages_equal_xpak_different() -> Result<()> {
        assert!(!packages_equal(
            &testdata(BINPKG)?,
            &testdata(BINPKG_DIFF_XPAK)?
        )?);

        Ok(())
    }

    #[test]
    fn packages_equal_tar_different() -> Result<()> {
        assert!(!packages_equal(
            &testdata(BINPKG)?,
            &testdata(BINPKG_DIFF_TAR)?
        )?);

        Ok(())
    }
}
