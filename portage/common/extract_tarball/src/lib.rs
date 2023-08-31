// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::{BufReader, Read},
    iter::Iterator,
    os::unix::prelude::OpenOptionsExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use tar::EntryType;

const PATCHELF_PATH: &str = "files/patchelf";
const EXECUTABLE_MASK: u32 = 0o111;
// All ELF files start with these 4 bytes.
const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

/// Contains arguments common to all binaries that extract from a tarball.
#[derive(Clone, Debug, clap::Args)]
pub struct CommonArgs {
    #[arg(
        long,
        help = "If true, when outputting an elf file, patch it to be able to \
    run outside of the SDK."
    )]
    pub patch_elf: bool,
}

fn get_patcher(patch_elf: bool) -> Result<Option<PathBuf>> {
    let r = runfiles::Runfiles::create()?;
    Ok(patch_elf.then(|| r.rlocation(PATCHELF_PATH)))
}

fn apply_patch(patcher: &Path, out_path: &Path) -> Result<()> {
    let mut command = Command::new("/lib64/ld-linux-x86-64.so.2");
    command.arg("--verify").arg(out_path);
    // Check whether the file is dynamically linked. Statically linked files will fail here.
    if processes::run(&mut command)?.success() {
        let mut command = Command::new(patcher);
        command
            .args([
                "--set-interpreter",
                "/tmp/cros_bazel_host_sysroot/lib64/ld-linux-x86-64.so.2",
            ])
            .args(["--add-rpath", "/tmp/cros_bazel_host_sysroot/lib"])
            .arg(out_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::piped());
        processes::run_and_check(&mut command)?;
    }

    Ok(())
}

fn is_shared_library(p: &Path) -> Result<bool> {
    let name = p
        .file_name()
        .context("Path must have file name")?
        .to_string_lossy();
    Ok(name.ends_with(".so") || name.contains(".so."))
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TarballContent {
    pub symlinks: Vec<PathBuf>,
    pub files: Vec<PathBuf>,
}

impl TarballContent {
    pub fn all_files(&self) -> impl Iterator<Item = &Path> {
        self.files
            .iter()
            .chain(self.symlinks.iter())
            .map(|p| p.as_path())
    }
}

/// Extracts from the specified tarball to out_dir.
/// If out_dir is specified, then we extract to the specified directory.
/// want_file is a mapping from the path within the archive to the path outside, relative to
/// out_dir.
/// For example:
/// * If we wanted to extract lib/foo to ${out_dir}/lib64/foo, we would map lib/foo to lib64/foo.
/// * If we wanted to not extract build_id, we would map path/to/.build_id/... to None.
/// Returns the contents of out_dir (or what it would have contained)
pub fn extract_tarball(
    archive: &mut tar::Archive<impl Sized + Read>,
    out_dir: Option<&Path>,
    patch_elf: bool,
    want_file: impl Fn(&Path) -> Result<Option<PathBuf>>,
) -> Result<TarballContent> {
    let patch_elf = get_patcher(patch_elf)?;

    let mut out_files: TarballContent = Default::default();
    for entry_result in archive.entries()? {
        let mut entry = entry_result?;
        let header = &entry.header();
        // Directories can't be extracted in a meaningful manner, because bazel doesn't deal with
        // directories. It can only deal with individual files.
        if header.entry_type() == tar::EntryType::Directory {
            continue;
        }
        let path = entry.path()?;
        let path = path.strip_prefix(".")?.to_path_buf();
        if let Some(relative_path) = want_file(&path)? {
            let out_path = Path::new("/").join(&relative_path);
            match header.entry_type() {
                EntryType::Regular => out_files.files.push(out_path),
                EntryType::Symlink | EntryType::Link => out_files.symlinks.push(out_path),
                _ => (),
            };

            if let Some(out_dir) = out_dir {
                let out_path = out_dir.join(&relative_path);
                let dir = out_path.parent().context("Path must have parent")?;
                std::fs::create_dir_all(&dir).with_context(|| "Failed to create {dir:?}")?;
                match header.entry_type() {
                    EntryType::Regular => {
                        let mode = header.mode()?;

                        // Peek at up to the first 4 bytes of the file to see if it's an ELF file.
                        // ELF files always have the first 4 bytes as ELF_MAGIC.
                        let mut content_header: Vec<u8> = vec![];
                        BufReader::new((&mut entry).take(4)).read_to_end(&mut content_header)?;
                        let is_elf: bool =
                            mode & EXECUTABLE_MASK != 0 && content_header.starts_with(&ELF_MAGIC);

                        let mut out_file = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .mode(mode)
                            .open(&out_path)?;
                        // Reading the first 4 bytes updated our seek position.
                        // Our position in the file is 4 bytes in. So we first need to copy the
                        // header before the rest of the file.
                        std::io::copy(&mut content_header.as_slice(), &mut out_file)?;
                        std::io::copy(&mut entry, &mut out_file)?;

                        if let Some(ref patcher) = patch_elf {
                            if is_elf && !is_shared_library(&path)? {
                                apply_patch(&patcher, &out_path)?
                            }
                        };
                    }
                    // Treat hard links the same as symlinks for now. We'll change this if this
                    // becomes a problem.
                    EntryType::Symlink | EntryType::Link => {
                        let dest = header
                            .link_name()?
                            .ok_or_else(|| anyhow!("Link name doesn't exist"))?;
                        let dest = if !dest.is_relative() {
                            pathdiff::diff_paths(&dest, &Path::new("/").join(&path))
                                .context("Failed to contsruct relative path")?
                        } else {
                            dest.to_path_buf()
                        };
                        std::os::unix::fs::symlink(&dest, &out_path).with_context(|| {
                            format!("Failed to symlink {out_path:?} to {dest:?}")
                        })?;
                    }
                    entry_type => bail!("Unsupported tar entry type: {:?}", entry_type),
                }
            }
        }
    }

    out_files.files.sort();
    out_files.symlinks.sort();
    Ok(out_files)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::os::unix::prelude::MetadataExt;

    use binarypackage::BinaryPackage;
    use fileutil::SafeTempDir;

    use super::*;

    const NANORC_SIZE: u64 = 11225;
    const NANO_SIZE: u64 = 225112;

    const BINARY_PKG_RUNFILE: &str = "cros/bazel/portage/common/testdata/nano.tbz2";

    fn binary_package() -> Result<BinaryPackage> {
        let r = runfiles::Runfiles::create()?;
        BinaryPackage::open(&r.rlocation(BINARY_PKG_RUNFILE))
    }

    fn extract(tmp_dir: &Path, patch_elf: bool) -> Result<TarballContent> {
        let mut bp = binary_package()?;
        let mapping = [
            (PathBuf::from("/bin/nano"), PathBuf::from("nano")),
            (PathBuf::from("/etc/nanorc"), PathBuf::from("nanorc")),
            (PathBuf::from("/bin/rnano"), PathBuf::from("rnano")),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        let content = extract_tarball(&mut bp.archive()?, Some(tmp_dir), patch_elf, |path| {
            Ok(mapping.get(&Path::new("/").join(path)).cloned())
        })?;

        Ok(content)
    }

    #[test]
    fn extracts_out_files() -> Result<()> {
        let tmp_dir = SafeTempDir::new()?;
        let content = extract(tmp_dir.path(), false)?;

        assert_eq!(
            content,
            TarballContent {
                files: vec![PathBuf::from("/nano"), PathBuf::from("/nanorc")],
                symlinks: vec![PathBuf::from("/rnano")],
            }
        );

        let nano_md = std::fs::metadata(tmp_dir.path().join("nano"))?;
        assert_eq!(nano_md.mode() & 0o777, 0o755);
        assert_eq!(nano_md.size(), NANO_SIZE);

        let nanorc_md = std::fs::metadata(tmp_dir.path().join("nanorc"))?;
        assert_eq!(nanorc_md.mode() & 0o777, 0o644);
        assert_eq!(nanorc_md.size(), NANORC_SIZE);

        let rnano_md = std::fs::symlink_metadata(tmp_dir.path().join("rnano"))?;
        assert!(rnano_md.is_symlink());
        assert_eq!(rnano_md.mode() & 0o777, 0o777);

        Ok(())
    }

    #[test]
    fn extracts_out_files_patched() -> Result<()> {
        let tmp_dir = SafeTempDir::new()?;
        let content = extract(tmp_dir.path(), true)?;

        assert_eq!(
            content,
            TarballContent {
                files: [PathBuf::from("/nano"), PathBuf::from("/nanorc")]
                    .into_iter()
                    .collect(),
                symlinks: [PathBuf::from("/rnano")].into_iter().collect()
            }
        );

        let nano_md = std::fs::metadata(tmp_dir.path().join("nano"))?;
        assert_eq!(nano_md.mode() & 0o777, 0o755);
        // It's quite difficult to get the interpreter / rpath and verify that
        // we have the extra entries.
        // We'll just assert that we've added data to the file.
        assert!(nano_md.size() > NANO_SIZE);

        let nanorc_md = std::fs::metadata(tmp_dir.path().join("nanorc"))?;
        assert_eq!(nanorc_md.mode() & 0o777, 0o644);
        assert_eq!(nanorc_md.size(), NANORC_SIZE);

        Ok(())
    }
}
