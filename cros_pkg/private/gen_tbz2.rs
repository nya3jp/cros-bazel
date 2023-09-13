// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use bytes::ByteOrder;
use bzip2::read::BzEncoder;
use bzip2::Compression;
use clap::Parser;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use version::Version;

const EAPI: &str = "7";
const KEYWORDS: &str = "*";
// Taken from an existing ebuild's environment.bz2 tag. Most of this probably
// isn't necessary.
const PORTAGE_FEATURES: &str = "\
assume-digests \
binpkg-hermetic \
binpkg-logs \
clean-logs \
distlocks \
fakeroot \
fixlafiles \
force-mirror \
nodoc \
noinfo \
noman \
parallel-install \
protect-owned \
sfperms \
userfetch \
userpriv \
usersync \
xattr";

#[derive(Parser)]
#[clap()]
struct Cli {
    #[arg(help = "Category (eg. 'chromeos-base')", long)]
    category: String,
    #[arg(help = "Package name (eg. 'goldctl')", long)]
    package_name: String,
    #[arg(help = "Package version (eg. '1.0.0-r1')", long)]
    version: String,
    #[arg(help = "Slot", long)]
    slot: String,
}

/// Usage: cat foo.tar | gen_tbz2 <clap args>
fn main() -> Result<()> {
    let args = Cli::parse();
    let version = Version::try_new(&args.version)?;
    let revision = version.revision().to_string();
    let p = format!("{}-{}", args.package_name, version.without_revision());
    let pf = format!("{}-{}", args.package_name, args.version);
    let env: [(&str, &str); 10] = [
        ("CATEGORY", &args.category),
        ("EAPI", EAPI),
        ("FEATURES", PORTAGE_FEATURES),
        ("KEYWORDS", KEYWORDS),
        ("P", &p),
        ("PF", &pf),
        ("PN", &args.package_name),
        ("PR", &revision),
        ("PVR", &args.version),
        ("SLOT", &args.slot),
    ];

    let mut env_contents: Vec<u8> = vec![];
    for (k, v) in env {
        writeln!(
            &mut env_contents,
            "declare -x {}={}",
            shell_escape::escape(k.into()),
            shell_escape::escape(v.into()),
        )?;
    }

    let mut env_bz2: Vec<u8> = vec![];
    std::io::copy(
        &mut BzEncoder::new(&env_contents[..], Compression::best()),
        &mut env_bz2,
    )?;

    let xpak: [(&str, &str); 6] = [
        ("CATEGORY", &args.category),
        ("EAPI", EAPI),
        ("FEATURES", PORTAGE_FEATURES),
        ("KEYWORDS", KEYWORDS),
        ("PF", &pf),
        ("SLOT", &args.slot),
    ];

    let args: BTreeMap<String, Vec<u8>> = xpak
        .iter()
        .map(|(k, v)| (k.to_string(), format!("{v}\n").as_bytes().to_vec()))
        .chain([("environment.bz2".to_string(), env_bz2)])
        .collect();

    gen_tbz2(&mut std::io::stdin(), args, &mut std::io::stdout())
}

/// Retrieves the length of the vector, in the format used by xpak.
fn len_bytes(v: &[u8]) -> Result<Vec<u8>> {
    let n: u32 = v.len().try_into()?;
    let mut out: Vec<u8> = vec![0; 4];
    bytes::BigEndian::write_u32(&mut out, n);
    Ok(out)
}

/// Combines an uncompressed tarball stream and an xpak into a writer.
fn gen_tbz2(r: impl Read, xpak: BTreeMap<String, Vec<u8>>, mut w: impl Write) -> Result<()> {
    // See https://www.mankier.com/5/xpak for the file format.
    let mut index: Vec<u8> = Vec::new();
    let mut data: Vec<u8> = Vec::new();
    for (k, v) in xpak {
        let k = k.as_bytes();
        index.extend(len_bytes(k)?);
        index.extend(k);
        index.extend(len_bytes(&data)?);
        index.extend(len_bytes(&v)?);
        data.extend(v);
    }

    let mut xpak: Vec<u8> = vec![];
    xpak.extend(b"XPAKPACK");
    xpak.extend(len_bytes(&index)?);
    xpak.extend(len_bytes(&data)?);
    xpak.extend(index);
    xpak.extend(data);
    xpak.extend(b"XPAKSTOP");

    std::io::copy(&mut zstd::stream::read::Encoder::new(r, 0)?, &mut w)?;
    w.write_all(&xpak)?;
    w.write_all(&len_bytes(&xpak)?)?;
    w.write_all(b"STOP")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use binarypackage::BinaryPackage;

    #[test]
    fn generates_readable_tbz2() -> Result<()> {
        let xpak: BTreeMap<String, Vec<u8>> = [
            ("CATEGORY".to_string(), b"app-editors".to_vec()),
            ("PF".to_string(), b"nano-6.4".to_vec()),
            ("SLOT".to_string(), b"0/0".to_vec()),
        ]
        .into();

        let want_content = b"content".to_vec();
        let mut out = tempfile::NamedTempFile::new()?;

        gen_tbz2(want_content.as_slice(), xpak.clone(), &mut out)?;

        let mut binpkg = BinaryPackage::open(out.path())?;

        let mut got_content: Vec<u8> = vec![];
        zstd::stream::read::Decoder::new(binpkg.new_tarball_reader()?)?
            .read_to_end(&mut got_content)?;

        assert_eq!(got_content, want_content);

        let want_xpak: BTreeMap<String, Vec<u8>> = binpkg.xpak().clone().into_iter().collect();
        assert_eq!(want_xpak, xpak);

        Ok(())
    }
}
