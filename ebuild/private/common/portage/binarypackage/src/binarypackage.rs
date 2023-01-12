// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::helpers::OutputFileSpec;
use crate::helpers::XpakSpec;
use anyhow::{anyhow, bail, Context, Result};
use bytes::ByteOrder;
use nix::fcntl::{fcntl, FcntlArg};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::SeekFrom::Start,
    io::{Read, Seek},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    os::unix::io::{AsRawFd, FromRawFd},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use tar::EntryType;

const CORRUPTED: &str = "corrupted .tbz2 file";

// See https://www.mankier.com/5/xpak for the format specification.
pub struct BinaryPackage {
    xpak_start: u64,
    size: u64,
    f: File,
}

impl BinaryPackage {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<BinaryPackage> {
        Self::_new(path.as_ref())
    }

    fn _new(path: &Path) -> Result<BinaryPackage> {
        let mut f = File::open(path)?;
        let md = std::fs::metadata(&path)?;
        let size = md.size();

        if size < 24 {
            bail!("corrupted .tbz2 file: size is too small")
        }
        expect_magic(&mut f, size - 4, "STOP").with_context(|| CORRUPTED)?;
        let bp_offset: u64 = u64::from(read_u32(&mut f, size - 8).with_context(|| CORRUPTED)?);
        let mut bp = Self {
            xpak_start: (size - 8 - bp_offset)
                .try_into()
                .with_context(|| "corrupted .tbz2 file: invalid bp offset")?,
            size,
            f,
        };
        bp.expect_magic(bp.size - 16, "XPAKSTOP")
            .with_context(|| CORRUPTED)?;
        bp.expect_magic(bp.xpak_start, "XPAKPACK")
            .with_context(|| CORRUPTED)?;

        Ok(bp)
    }

    pub fn new_tarball_reader(&self) -> Result<std::io::Take<File>> {
        let new_fd = fcntl(
            self.f.as_raw_fd(),
            FcntlArg::F_DUPFD_CLOEXEC(self.f.as_raw_fd()),
        )?;
        let mut f = unsafe { File::from_raw_fd(new_fd) };
        f.rewind()?;
        Ok(f.take(self.xpak_start))
    }

    pub fn merge<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        // Note that at the moment, ownership is not retained.
        // --same-owner should fix that, but:
        // 1) it can only be run with sudo.
        // 2) If we generate bazel output files with strange ownership, bazel won't
        // have permissions to clean it up.
        // When we write to an image, we'll need to do some work to preserve metadata.
        let mut child = Command::new("tar")
            .args([
                "--zstd",
                "--keep-old-files",
                "--same-permissions",
                "-xf",
                "-",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .current_dir(dir)
            .spawn()?;

        match child.stdin {
            None => bail!("No stdin"),
            Some(ref mut stdin) => std::io::copy(&mut self.new_tarball_reader()?, stdin)?,
        };

        if !child.wait_with_output()?.status.success() {
            bail!("failed to extract tarball - maybe multiple packages attempt to define the same file")
        }
        Ok(())
    }

    pub fn xpak(&mut self) -> Result<HashMap<String, Vec<u8>>> {
        let index_len = u64::from(self.read_u32(self.xpak_start + 8)?);
        let data_len = u64::from(self.read_u32(self.xpak_start + 12)?);
        let index_start = self.xpak_start + 16;
        let data_start = index_start + index_len;
        if data_start + data_len != self.size - 16 {
            bail!("corrupted .tbz2 file: data length inconsistency")
        }

        let mut xpak: HashMap<String, Vec<u8>> = HashMap::new();
        let mut index_pos = index_start;
        while index_pos < data_start {
            let name_len = u64::from(self.read_u32(index_pos)?);
            index_pos += 4;
            let mut name: String = String::new();
            (&self.f).take(name_len).read_to_string(&mut name)?;
            if name.len() != name_len.try_into()? {
                bail!("Got '{name}', want a name of length {name_len}")
            }
            index_pos += name_len;
            let data_offset = u64::from(self.read_u32(index_pos)?);
            index_pos += 4;
            let data_len = u64::from(self.read_u32(index_pos)?);
            index_pos += 4;

            self.f.seek(Start(data_start + data_offset))?;
            let mut data = Vec::new();
            (&self.f).take(data_len).read_to_end(&mut data)?;
            if data.len() != data_len.try_into()? {
                bail!(
                    "Got a buffer of length {}, want length {}",
                    data.len(),
                    data_len
                );
            }

            xpak.insert(name, data);
        }
        Ok(xpak)
    }

    pub fn extract_xpak_files(&mut self, specs: &[XpakSpec]) -> Result<()> {
        if specs.len() == 0 {
            return Ok(());
        }

        let headers = self.xpak()?;
        for spec in specs.iter() {
            let v: Vec<u8> = Vec::new();
            let contents = headers
                .get(&spec.xpak_header)
                .or(if spec.optional { Some(&v) } else { None })
                .ok_or(anyhow!("XPAK key {} not found in header", spec.xpak_header))?;
            std::fs::write(&spec.target_path, contents)?;
        }
        Ok(())
    }

    pub fn extract_out_files(&mut self, specs: &[OutputFileSpec]) -> Result<()> {
        if specs.len() == 0 {
            return Ok(());
        }

        let mut file_map: HashMap<String, &PathBuf> = HashMap::new();

        for spec in specs.iter() {
            // We might request for /bin/nano, but it's stored in the archive as ./bin/nano
            file_map.insert(format!(".{}", spec.inside_path), &spec.target_path);
        }

        let mut archive = tar::Archive::new(zstd::stream::read::Decoder::new(
            self.new_tarball_reader()?,
        )?);

        for entry_result in archive.entries()?.into_iter() {
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
                                .ok_or(anyhow!("Link name doesn't exist"))?;
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

    fn read_u32(&mut self, offset: u64) -> Result<u32> {
        read_u32(&mut self.f, offset)
    }

    fn expect_magic(&mut self, offset: u64, want: &str) -> Result<()> {
        expect_magic(&mut self.f, offset, want)
    }
}

fn read_u32(f: &mut File, offset: u64) -> Result<u32> {
    f.seek(Start(offset))?;
    let mut buffer = [0_u8; std::mem::size_of::<u32>()];
    f.read_exact(&mut buffer)?;
    Ok(bytes::BigEndian::read_u32(&mut buffer))
}

fn expect_magic(f: &mut File, offset: u64, want: &str) -> Result<()> {
    f.seek(Start(offset))?;
    let mut got: String = "".to_string();
    f.take(want.len() as u64).read_to_string(&mut got)?;
    if got != want {
        bail!("Bad magic: got {got}, want {want}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const NANORC_SIZE: u64 = 11225;
    const NANO_SIZE: u64 = 225112;

    const BINARY_PKG_RUNFILE: &str =
        "chromiumos/bazel/ebuild/private/common/portage/binarypackage/testdata/nano.tbz2";

    // This is the contents of the tarball from the binary package, extracted with
    // qtbz2. We are unable to use the qtbz2 binary in bazel, as this would require
    // first being able to extract the qtbz2 binary from a package.
    const TARBALL_RUNFILE: &str =
        "chromiumos/bazel/ebuild/private/common/portage/binarypackage/testdata/nano.tar.bz2";

    fn binary_package() -> Result<BinaryPackage> {
        let r = runfiles::Runfiles::create()?;
        BinaryPackage::new(r.rlocation(BINARY_PKG_RUNFILE))
    }

    #[test]
    fn parse_xpak_metadata() -> Result<()> {
        let mut bp = binary_package()?;
        let xpak = bp.xpak()?;
        assert_eq!(
            xpak.get("CATEGORY")
                .map(|x| std::str::from_utf8(x).unwrap()),
            Some("app-editors\n")
        );
        assert_eq!(
            xpak.get("PF").map(|x| std::str::from_utf8(x).unwrap()),
            Some("nano-6.4\n")
        );
        assert_eq!(
            xpak.get("repository")
                .map(|x| std::str::from_utf8(x).unwrap()),
            Some("portage-stable\n")
        );
        Ok(())
    }

    #[test]
    fn valid_tarball() -> Result<()> {
        let r = runfiles::Runfiles::create()?;
        let bp = binary_package()?;

        let mut got: Vec<u8> = Vec::new();
        bp.new_tarball_reader()?.read_to_end(&mut got)?;

        let mut want: Vec<u8> = Vec::new();
        File::open(r.rlocation(TARBALL_RUNFILE))?.read_to_end(&mut want)?;

        assert_eq!(got.len(), want.len());
        // Don't use assert_eq, since the files are massive and it'd print out bytes to stderr.
        assert!(got == want);

        Ok(())
    }

    #[test]
    fn can_merge() -> Result<()> {
        let bp = binary_package()?;

        let tmp_dir = tempfile::tempdir()?;
        let out_dir = tmp_dir.path();
        std::fs::create_dir(out_dir.join("bin"))?;
        // Put an arbitrary file in the tree to ensure it can merge with
        // existing files.
        std::fs::write(out_dir.join("bin/vim"), "")?;

        bp.merge(out_dir)?;

        let nano = std::fs::metadata(out_dir.join("bin/nano"))?;
        assert_eq!(nano.mode() & 0o777, 0o755);
        assert_eq!(nano.size(), NANO_SIZE);

        let nanorc = std::fs::metadata(out_dir.join("etc/nanorc"))?;
        assert_eq!(nanorc.mode() & 0o777, 0o644);
        assert_eq!(nanorc.size(), NANORC_SIZE);

        Ok(())
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

        bp.extract_xpak_files(&[category.clone(), optional_not_present.clone()])?;
        assert_eq!(
            std::fs::read_to_string(category.target_path)?,
            "app-editors\n"
        );
        assert_eq!(
            std::fs::read_to_string(optional_not_present.target_path)?,
            ""
        );

        assert!(bp.extract_xpak_files(&[required_not_present]).is_err());

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

        bp.extract_out_files(&[nano.clone(), nanorc.clone()])?;
        let nano_md = std::fs::metadata(nano.target_path)?;
        assert_eq!(nano_md.mode() & 0o777, 0o755);
        assert_eq!(nano_md.size(), NANO_SIZE);

        let nanorc_md = std::fs::metadata(nanorc.target_path)?;
        assert_eq!(nanorc_md.mode() & 0o777, 0o644);
        assert_eq!(nanorc_md.size(), NANORC_SIZE);

        Ok(())
    }
}
