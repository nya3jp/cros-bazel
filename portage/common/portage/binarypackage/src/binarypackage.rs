// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, ensure, Context, Result};
use bytes::ByteOrder;
use processes::locate_system_binary;
use runfiles::Runfiles;
use std::{
    collections::HashMap,
    fs::File,
    io::SeekFrom::Start,
    io::{Read, Seek},
    os::unix::fs::MetadataExt,
    path::Path,
    process::{Command, Stdio},
};

/// Works with Portage binary package files (.tbz2).
///
/// See https://www.mankier.com/5/xpak for the format specification.
pub struct BinaryPackage {
    file: File,
    xpak_start: u64,
    xpak_len: u64,
    xpak: HashMap<String, Vec<u8>>,
    category_pf: String,
}

impl BinaryPackage {
    /// Opens a Portage binary package file.
    pub fn open(path: &Path) -> Result<Self> {
        let mut file = File::open(path)?;
        let metadata = std::fs::metadata(path)?;
        let size = metadata.size();

        if size < 24 {
            bail!("corrupted .tbz2 file: size is too small")
        }

        const CORRUPTED: &str = "Corrupted .tbz2 file";

        expect_magic(&mut file, size - 4, "STOP").context(CORRUPTED)?;
        expect_magic(&mut file, size - 16, "XPAKSTOP").context(CORRUPTED)?;

        let xpak_offset: u64 = u64::from(read_u32(&mut file, size - 8).context(CORRUPTED)?);
        let xpak_start = size - 8 - xpak_offset;

        expect_magic(&mut file, xpak_start, "XPAKPACK").context(CORRUPTED)?;

        let xpak = parse_xpak(&mut file, xpak_start, size)?;

        let category = std::str::from_utf8(
            xpak.get("CATEGORY")
                .with_context(|| "Binary package missing CATEGORY")?,
        )?
        .trim();
        let pf = std::str::from_utf8(
            xpak.get("PF")
                .with_context(|| "Binary package missing PF")?,
        )?
        .trim();
        let category_pf = format!("{category}/{pf}");

        Ok(Self {
            file,
            xpak_start,
            xpak_len: size - xpak_start,
            xpak,
            category_pf,
        })
    }

    /// Returns the XPAK key-value map.
    pub fn xpak(&self) -> &HashMap<String, Vec<u8>> {
        &self.xpak
    }

    /// Returns the string combining CATEGORY and PF, e.g. "sys-apps/attr-2.5.1".
    pub fn category_pf(&self) -> &str {
        &self.category_pf
    }

    /// Returns a tarball reader.
    pub fn new_tarball_reader(&mut self) -> Result<impl Sized + Read + '_> {
        self.file.rewind()?;
        Ok((&mut self.file).take(self.xpak_start))
    }

    /// Returns a xpak reader.
    pub fn new_xpak_reader(&mut self) -> Result<impl Sized + Read + '_> {
        self.file.seek(Start(self.xpak_start))?;
        Ok((&mut self.file).take(self.xpak_len))
    }

    /// Returns a tar archive.
    pub fn archive(&mut self) -> Result<tar::Archive<impl Sized + Read + '_>> {
        Ok(tar::Archive::new(zstd::stream::read::Decoder::new(
            self.new_tarball_reader()?,
        )?))
    }

    /// Extracts the contents of the archive to the specified directory.
    /// It uses fakefs to apply ownership information.
    pub fn extract_image(&mut self, output_dir: &Path) -> Result<()> {
        let runfiles = Runfiles::create()?;
        let fakefs_path = runfiles.rlocation("cros/bazel/portage/bin/fakefs/fakefs_/fakefs");
        let zstd_path = runfiles.rlocation("zstd/zstd");

        let mut tarball = self.new_tarball_reader()?;

        let mut child = Command::new(fakefs_path)
            .arg(locate_system_binary("tar")?)
            .arg("-x")
            .arg("--same-permissions")
            .arg("--same-owner")
            .arg("-I")
            .arg(&zstd_path)
            .arg("-C")
            .arg(output_dir)
            .stdin(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().expect("stdin must be piped");
        std::io::copy(&mut tarball, &mut stdin)?;
        drop(stdin);

        let status = child.wait()?;
        ensure!(status.success(), "tar failed: {:?}", status);

        Ok(())
    }
}

fn read_u32(f: &mut File, offset: u64) -> Result<u32> {
    f.seek(Start(offset))?;
    let mut buffer = [0_u8; std::mem::size_of::<u32>()];
    f.read_exact(&mut buffer)?;
    Ok(bytes::BigEndian::read_u32(&buffer))
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

fn parse_xpak(file: &mut File, xpak_start: u64, size: u64) -> Result<HashMap<String, Vec<u8>>> {
    let index_len = u64::from(read_u32(file, xpak_start + 8)?);
    let data_len = u64::from(read_u32(file, xpak_start + 12)?);
    let index_start = xpak_start + 16;
    let data_start = index_start + index_len;
    if data_start + data_len != size - 16 {
        bail!("corrupted .tbz2 file: data length inconsistency")
    }

    let mut xpak: HashMap<String, Vec<u8>> = HashMap::new();
    let mut index_pos = index_start;
    while index_pos < data_start {
        let name_len = u64::from(read_u32(file, index_pos)?);
        index_pos += 4;
        let mut name: String = String::new();
        file.take(name_len).read_to_string(&mut name)?;
        if name.len() != name_len.try_into()? {
            bail!("Got '{name}', want a name of length {name_len}")
        }
        index_pos += name_len;
        let data_offset = u64::from(read_u32(file, index_pos)?);
        index_pos += 4;
        let data_len = u64::from(read_u32(file, index_pos)?);
        index_pos += 4;

        file.seek(Start(data_start + data_offset))?;
        let mut data = Vec::new();
        file.take(data_len).read_to_end(&mut data)?;
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

#[cfg(test)]
mod tests {
    use fileutil::SafeTempDir;

    use super::*;

    fn binary_package() -> Result<BinaryPackage> {
        let runfiles = Runfiles::create()?;
        BinaryPackage::open(&runfiles.rlocation(
            "cros/bazel/portage/common/portage/binarypackage/testdata/binpkg-test-1.2.3.tbz2",
        ))
    }

    #[test]
    fn xpak() -> Result<()> {
        let bp = binary_package()?;
        let xpak = bp.xpak();
        assert_eq!(
            xpak.get("CATEGORY")
                .map(|x| std::str::from_utf8(x).unwrap()),
            Some("sys-apps\n")
        );
        assert_eq!(
            xpak.get("PF").map(|x| std::str::from_utf8(x).unwrap()),
            Some("binpkg-test-1.2.3\n")
        );
        assert_eq!(
            xpak.get("repository")
                .map(|x| std::str::from_utf8(x).unwrap()),
            Some("chromiumos\n")
        );
        Ok(())
    }

    #[test]
    fn category_pf() -> Result<()> {
        let bp = binary_package()?;
        assert_eq!("sys-apps/binpkg-test-1.2.3", bp.category_pf());
        Ok(())
    }

    #[test]
    fn valid_tarball() -> Result<()> {
        let mut bp = binary_package()?;

        // Just ensure that tar accepts the tarball without any error.
        let runfiles = Runfiles::create()?;
        let zstd_path = runfiles.rlocation("zstd/zstd");
        let mut tar = Command::new(locate_system_binary("tar")?)
            .arg("-I")
            .arg(&zstd_path)
            .arg("-t")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()?;
        let mut stdin = tar.stdin.take().expect("stdin must be piped");
        std::io::copy(&mut bp.new_tarball_reader()?, &mut stdin)?;
        drop(stdin);
        let status = tar.wait()?;
        assert!(status.success(), "tar failed: {:?}", status);

        Ok(())
    }

    #[test]
    fn extract_image() -> Result<()> {
        let mut bp = binary_package()?;

        let temp_dir = SafeTempDir::new()?;
        let temp_dir = temp_dir.path();

        bp.extract_image(temp_dir)?;

        let hello_path = temp_dir.join("usr/bin/hello");
        assert!(hello_path.try_exists()?);

        // File ownership info should be available via fakefs.
        let runfiles = Runfiles::create()?;
        let fakefs_path = runfiles.rlocation("cros/bazel/portage/bin/fakefs/fakefs_/fakefs");
        let output = Command::new(fakefs_path)
            .arg("stat")
            .arg("--format=%u:%g")
            .arg(&hello_path)
            .stderr(Stdio::inherit())
            .output()?;
        assert!(output.status.success(), "stat failed: {:?}", output.status);
        let stdout = String::from_utf8(output.stdout)?;
        assert_eq!(stdout.trim(), "123:234");

        Ok(())
    }
}
