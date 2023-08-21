// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use binarypackage::BinaryPackage;
use md5::{Digest, Md5};
use walkdir::WalkDir;

/// Computes the VDB directory path for a package.
pub fn get_vdb_dir(root_dir: &Path, cpf: &str) -> PathBuf {
    root_dir.join("var/db/pkg").join(cpf)
}

/// Creates an initial VDB directory for a package.
///
/// You need to generate CONTENTS file to finish the VDB directory creation.
/// See [`generate_contents_file`].
pub fn create_initial_vdb(vdb_dir: &Path, package: &BinaryPackage) -> Result<()> {
    std::fs::create_dir_all(vdb_dir).with_context(|| format!("mkdir {}", vdb_dir.display()))?;

    // Extract xpak.
    for (key, value) in package.xpak().iter() {
        std::fs::write(vdb_dir.join(key), value)?;
    }

    // Create additional files that are not contained in XPAK but created by
    // Portage.
    std::fs::write(vdb_dir.join("COUNTER"), "0")?;

    // TODO: Do we need to create INSTALL_MASK?

    Ok(())
}

/// Computes the MD5 hash of a file in a hexadecimal format.
fn compute_md5_hash(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Md5::new();
    std::io::copy(&mut file, &mut hasher)?;
    Ok(hex::encode(hasher.finalize()))
}

/// Generates `CONTENTS` file from an image directory.
pub fn generate_vdb_contents<W: Write>(mut w: W, image_dir: &Path) -> Result<()> {
    for entry in WalkDir::new(image_dir).min_depth(1).sort_by_file_name() {
        let entry = entry?;
        let relative_path = entry.path().strip_prefix(image_dir)?;
        let file_type = entry.file_type();
        if file_type.is_dir() {
            // "dir <path>"
            writeln!(&mut w, "dir {}", relative_path.to_string_lossy())?;
        } else if file_type.is_file() {
            let hash = compute_md5_hash(entry.path())?;
            // "obj <path> <md5> <timestamp>"
            writeln!(&mut w, "obj {} {} 0", relative_path.to_string_lossy(), hash)?;
        } else if file_type.is_symlink() {
            let target = std::fs::read_link(entry.path())?;
            // "sym <path> -> <target> <timestamp>"
            writeln!(
                &mut w,
                "sym {} -> {} 0",
                relative_path.to_string_lossy(),
                target.to_string_lossy()
            )?;
        } else {
            bail!("Unknown file type: {:?}", file_type);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use testutil::compare_with_golden_data;

    use super::*;

    fn open_test_binary_package() -> Result<BinaryPackage> {
        BinaryPackage::open(Path::new(
            "bazel/portage/common/portage/vdb/testdata/vdb-test-1.2.3.tbz2",
        ))
    }

    #[test]
    fn test_get_vdb_dir() {
        let vdb_dir = get_vdb_dir(Path::new("/build/foo"), "aaa/bbb-1.2.3");
        assert_eq!(
            vdb_dir,
            PathBuf::from("/build/foo/var/db/pkg/aaa/bbb-1.2.3")
        );
    }

    #[test]
    fn test_create_initial_vdb() -> Result<()> {
        let package = open_test_binary_package()?;

        let vdb_dir = TempDir::new()?;
        let vdb_dir = vdb_dir.path();
        create_initial_vdb(vdb_dir, &package)?;

        let golden_dir = Path::new("bazel/portage/common/portage/vdb/testdata/golden/vdb");
        compare_with_golden_data(vdb_dir, golden_dir)?;

        Ok(())
    }

    #[test]
    fn test_generate_vdb_contents() -> Result<()> {
        let mut package = open_test_binary_package()?;

        let image_dir = TempDir::new()?;
        let image_dir = image_dir.path();
        package.extract_image(image_dir)?;

        let mut contents: Vec<u8> = Vec::new();
        generate_vdb_contents(&mut contents, image_dir)?;
        let contents = String::from_utf8(contents)?;

        assert_eq!(
            contents,
            r"dir usr
dir usr/bin
obj usr/bin/helloworld d9ee44d59390c7097f20a0ec1c449048 0
sym usr/bin/helloworld.symlink -> helloworld 0
"
        );

        Ok(())
    }
}
