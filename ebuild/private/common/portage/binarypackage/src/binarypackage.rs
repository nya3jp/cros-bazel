// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use bytes::ByteOrder;
use std::{
    collections::HashMap,
    fs::File,
    io::SeekFrom::Start,
    io::{Read, Seek},
    os::unix::fs::MetadataExt,
    path::Path,
};

/// Works with Portage binary package files (.tbz2).
///
/// See https://www.mankier.com/5/xpak for the format specification.
pub struct BinaryPackage {
    file: File,
    xpak_start: u64,
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
    pub fn new_tarball_reader(&mut self) -> Result<std::io::Take<&mut File>> {
        self.file.rewind()?;
        Ok((&mut self.file).take(self.xpak_start))
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
    use super::*;

    const BINARY_PKG_RUNFILE: &str =
        "cros/bazel/ebuild/private/common/portage/binarypackage/testdata/nano.tbz2";

    // This is the contents of the tarball from the binary package, extracted with
    // qtbz2. We are unable to use the qtbz2 binary in bazel, as this would require
    // first being able to extract the qtbz2 binary from a package.
    const TARBALL_RUNFILE: &str =
        "cros/bazel/ebuild/private/common/portage/binarypackage/testdata/nano.tar.bz2";

    fn binary_package() -> Result<BinaryPackage> {
        let r = runfiles::Runfiles::create()?;
        BinaryPackage::open(&r.rlocation(BINARY_PKG_RUNFILE))
    }

    #[test]
    fn xpak() -> Result<()> {
        let bp = binary_package()?;
        let xpak = bp.xpak();
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
    fn category_pf() -> Result<()> {
        let bp = binary_package()?;
        assert_eq!("app-editors/nano-6.4", bp.category_pf());
        Ok(())
    }

    #[test]
    fn valid_tarball() -> Result<()> {
        let r = runfiles::Runfiles::create()?;
        let mut bp = binary_package()?;

        let mut got: Vec<u8> = Vec::new();
        bp.new_tarball_reader()?.read_to_end(&mut got)?;

        let mut want: Vec<u8> = Vec::new();
        File::open(r.rlocation(TARBALL_RUNFILE))?.read_to_end(&mut want)?;

        assert_eq!(got.len(), want.len());
        // Don't use assert_eq, since the files are massive and it'd print out bytes to stderr.
        assert!(got == want);

        Ok(())
    }
}
