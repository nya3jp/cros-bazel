// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod specs;

use anyhow::{anyhow, bail, Result};
use binarypackage::BinaryPackage;
use clap::Parser;
use cliutil::cli_main;
use specs::{OutputFileSpec, XpakSpec};
use std::{
    collections::HashMap,
    fs::OpenOptions,
    os::unix::prelude::OpenOptionsExt,
    path::{Path, PathBuf},
    process::ExitCode,
};
use tar::EntryType;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Cli {
    #[arg(long, required = true)]
    binpkg: PathBuf,

    #[arg(
        long,
        help = "<inside path>=<outside path>: Extracts a file from the binpkg and writes it to the outside path"
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
    if specs.is_empty() {
        return Ok(());
    }

    let xpak = pkg.xpak();

    for spec in specs.iter() {
        let v: Vec<u8> = Vec::new();
        let contents = xpak
            .get(&spec.xpak_header)
            .or(if spec.optional { Some(&v) } else { None })
            .ok_or_else(|| anyhow!("XPAK key {} not found in header", spec.xpak_header))?;
        std::fs::write(&spec.target_path, contents)?;
    }
    Ok(())
}

fn extract_out_files(pkg: &mut BinaryPackage, specs: &[OutputFileSpec]) -> Result<()> {
    if specs.is_empty() {
        return Ok(());
    }

    let mut file_map: HashMap<String, &PathBuf> = HashMap::new();

    for spec in specs.iter() {
        // We might request for /bin/nano, but it's stored in the archive as ./bin/nano
        file_map.insert(format!(".{}", spec.inside_path), &spec.target_path);
    }

    let mut archive =
        tar::Archive::new(zstd::stream::read::Decoder::new(pkg.new_tarball_reader()?)?);

    for entry_result in archive.entries()? {
        let mut entry = entry_result?;
        let header = &entry.header();
        let path = entry.path()?;
        match file_map.remove_entry(&path.to_string_lossy().to_string()) {
            None => continue,
            Some((_, out_path)) => {
                match header.entry_type() {
                    EntryType::Regular => {
                        let mode = header.mode()?;
                        std::io::copy(
                            &mut entry,
                            &mut OpenOptions::new()
                                .write(true)
                                .create(true)
                                .mode(mode)
                                .open(out_path)?,
                        )?;
                    }
                    EntryType::Symlink => {
                        let dest = header
                            .link_name()?
                            .ok_or_else(|| anyhow!("Link name doesn't exist"))?;
                        // bazel only supports relative symlinks that point to existing files.
                        // Let's limit this to symlinks that point to files in the same
                        // directory for now.
                        if !dest.is_relative() || dest.parent() != Some(Path::new("")) {
                            bail!(
                                "symlinks paths separators are currently unsupported {:?} -> {:?}",
                                path,
                                dest
                            )
                        }
                        std::os::unix::fs::symlink(out_path, dest)?;
                    }
                    entry_type => bail!("Unsupported tar entry type: {:?}", entry_type),
                }
            }
        }
    }

    if !file_map.is_empty() {
        bail!("Failed to extract {file_map:?}")
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
    cli_main(do_main)
}

#[cfg(test)]
mod tests {
    use std::os::unix::prelude::MetadataExt;

    use super::*;

    const NANORC_SIZE: u64 = 11225;
    const NANO_SIZE: u64 = 225112;

    const BINARY_PKG_RUNFILE: &str =
        "cros/bazel/ebuild/private/cmd/extract_interface/testdata/nano.tbz2";

    fn binary_package() -> Result<BinaryPackage> {
        let r = runfiles::Runfiles::create()?;
        BinaryPackage::open(&r.rlocation(BINARY_PKG_RUNFILE))
    }

    #[test]
    fn extracts_xpak_files() -> Result<()> {
        let mut bp = binary_package()?;

        let tmp_dir = tempfile::tempdir()?;

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
        let tmp_dir = tempfile::tempdir()?;

        let nano = OutputFileSpec {
            inside_path: "/bin/nano".to_string(),
            target_path: tmp_dir.path().join("nano"),
        };
        let nanorc = OutputFileSpec {
            inside_path: "/etc/nanorc".to_string(),
            target_path: tmp_dir.path().join("nanorc"),
        };

        extract_out_files(&mut bp, &[nano.clone(), nanorc.clone()])?;
        let nano_md = std::fs::metadata(nano.target_path)?;
        assert_eq!(nano_md.mode() & 0o777, 0o755);
        assert_eq!(nano_md.size(), NANO_SIZE);

        let nanorc_md = std::fs::metadata(nanorc.target_path)?;
        assert_eq!(nanorc_md.mode() & 0o777, 0o644);
        assert_eq!(nanorc_md.size(), NANORC_SIZE);

        Ok(())
    }
}
