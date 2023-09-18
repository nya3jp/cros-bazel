// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod specs;

use anyhow::{bail, Context, Result};
use binarypackage::BinaryPackage;
use clap::Parser;
use cliutil::cli_main;
use specs::{OutputFileSpec, XpakSpec};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Cli {
    #[arg(long, required = true)]
    binpkg: PathBuf,

    #[arg(
        long,
        help = "<inside path>=<outside path>: Extracts a file from the binpkg and writes it to \
        the outside path"
    )]
    output_file: Vec<OutputFileSpec>,

    #[arg(
        long,
        help = "<XPAK key>=[?]<output file>: Write the XPAK key from the binpkg to the \
    specified file. If =? is used then an empty file is created if XPAK key doesn't exist."
    )]
    xpak: Vec<XpakSpec>,
}

fn extract_xpak_files(pkg: &mut BinaryPackage, specs: &[XpakSpec]) -> Result<()> {
    let xpak = pkg.xpak();

    for spec in specs.iter() {
        let v: Vec<u8> = Vec::new();
        let contents = xpak
            .get(&spec.xpak_header)
            .or(if spec.optional { Some(&v) } else { None })
            .with_context(|| format!("XPAK key {} not found in header", spec.xpak_header))?;
        std::fs::write(&spec.target_path, contents)?;
    }
    Ok(())
}

fn extract_out_files(binpkg: &mut BinaryPackage, specs: &[OutputFileSpec]) -> Result<()> {
    if specs.is_empty() {
        return Ok(());
    }

    let want_files: BTreeMap<&Path, &Path> = specs
        .iter()
        .map(|spec| (spec.inside_path.as_path(), spec.target_path.as_path()))
        .collect();
    let content =
        common_extract_tarball::extract_tarball(&mut binpkg.archive()?, Path::new("."), |path| {
            Ok(want_files
                .get(Path::new("/").join(path).as_path())
                .map(|p| p.to_path_buf()))
        })?;

    let got_paths: BTreeSet<&Path> = content.all_files().collect();

    for (tar_path, extracted_path) in want_files {
        if !got_paths.contains(Path::new("/").join(extracted_path).as_path()) {
            bail!("Failed to extract {tar_path:?} from archive")
        }
    }

    Ok(())
}

fn extract_files(
    bin_pkg: &Path,
    xpak_specs: &[XpakSpec],
    output_file_specs: &[OutputFileSpec],
) -> Result<()> {
    if xpak_specs.is_empty() && output_file_specs.is_empty() {
        return Ok(());
    }
    let mut pkg = BinaryPackage::open(bin_pkg)?;
    extract_xpak_files(&mut pkg, xpak_specs)?;
    extract_out_files(&mut pkg, output_file_specs)?;
    Ok(())
}

fn do_main() -> Result<()> {
    let args = Cli::parse();
    extract_files(&args.binpkg, &args.xpak, &args.output_file)
}

fn main() -> ExitCode {
    cli_main(do_main, Default::default())
}

#[cfg(test)]
mod tests {
    use std::os::unix::prelude::MetadataExt;

    use fileutil::SafeTempDir;

    use super::*;

    const NANO_SIZE: u64 = 225112;

    const BINARY_PKG_RUNFILE: &str = "cros/bazel/portage/common/testdata/nano.tbz2";

    fn binary_package() -> Result<BinaryPackage> {
        let r = runfiles::Runfiles::create()?;
        BinaryPackage::open(&r.rlocation(BINARY_PKG_RUNFILE))
    }

    #[test]
    fn extracts_xpak_files() -> Result<()> {
        let mut bp = binary_package()?;

        let tmp_dir = SafeTempDir::new()?;

        let category = XpakSpec {
            xpak_header: "CATEGORY".to_string(),
            target_path: tmp_dir.path().join("category"),
            optional: false,
        };
        let optional_not_present = XpakSpec {
            xpak_header: "NOT_PRESENT".to_string(),
            target_path: tmp_dir.path().join("not_present_optional"),
            optional: true,
        };
        let required_not_present = XpakSpec {
            xpak_header: "NOT_PRESENT".to_string(),
            target_path: tmp_dir.path().join("not_present_required"),
            optional: false,
        };

        extract_xpak_files(&mut bp, &[category.clone(), optional_not_present.clone()])?;
        assert_eq!(
            std::fs::read_to_string(category.target_path)?,
            "app-editors\n"
        );
        assert_eq!(
            std::fs::read_to_string(optional_not_present.target_path)?,
            ""
        );

        assert!(extract_xpak_files(&mut bp, &[required_not_present]).is_err());

        Ok(())
    }

    #[test]
    fn extracts_out_files() -> Result<()> {
        let mut bp = binary_package()?;
        let tmp_dir = SafeTempDir::new()?;

        let nano = OutputFileSpec {
            inside_path: PathBuf::from("/bin/nano"),
            target_path: tmp_dir.path().join("nano"),
        };

        extract_out_files(&mut bp, &[nano.clone()])?;
        let nano_md = std::fs::metadata(nano.target_path)?;
        assert_eq!(nano_md.mode() & 0o777, 0o755);
        assert_eq!(nano_md.size(), NANO_SIZE);

        Ok(())
    }
}
