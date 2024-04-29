// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::HashSet,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{bail, ensure, Context, Result};
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

/// Creates an sparse VDB directory for a package.
///
/// This provides a database entry with enough metadata for `has_version` to
/// function correctly.
///
/// We omit the files instead of clearing them so that when
/// `fast_install_packages` layers the installed contents layer on top of the
/// staged contents layer we don't hide the real files.
pub fn create_sparse_vdb(vdb_dir: &Path, package: &BinaryPackage) -> Result<()> {
    std::fs::create_dir_all(vdb_dir).with_context(|| format!("mkdir {}", vdb_dir.display()))?;

    // These are the keys we need so that `has_version` can properly support
    // a full dependency atom. i.e.,
    //     * dev-python/six[python_targets_python3_8(-),python_single_target_python3_8(+)]
    //     * dev-libs/openssl:3
    //     * dev-libs/openssl::portage-stable
    //     * sys-devel/glibc[crosscompile_opts_headers-only]
    let mut keys = HashSet::from(["EAPI", "SLOT", "IUSE", "USE", "repository"]);

    // Extract xpak.
    for (key, value) in package.xpak().iter() {
        // Drop the sub-slot since cros-workon packages normally contain the
        // revision number in the sub-slot.
        let override_value = if key == "SLOT" {
            let slot = String::from_utf8(value.clone())
                .with_context(|| format!("Slot is not UTF-8: {:?}", value))?;
            let primary_slot = slot.trim_end().split('/').next().unwrap();

            Some(format!("{}/{}\n", primary_slot, primary_slot).into_bytes())
        } else {
            None
        };

        if keys.remove(key.as_str()) {
            std::fs::write(vdb_dir.join(key), override_value.as_ref().unwrap_or(value))?;
        }
    }

    ensure!(
        keys.is_empty(),
        "Package is missing the following keys: {keys:?}"
    );

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

    // These unit tests need to run in an user namespace so that the current process UID/GID are 0.
    // Otherwise `BinaryPackage::extract_image` fails because it cannot chown symlinks.
    #[cfg(test)]
    #[used]
    #[link_section = ".init_array"]
    static _CTOR: extern "C" fn() = ::testutil::ctor_enter_mount_namespace;

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
        package.extract_image(image_dir, true)?;

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

    #[test]
    fn test_create_sparse_vdb() -> Result<()> {
        let package = open_test_binary_package()?;

        let vdb_dir = TempDir::new()?;
        let vdb_dir = vdb_dir.path();
        create_sparse_vdb(vdb_dir, &package)?;

        let golden_dir = Path::new("bazel/portage/common/portage/vdb/testdata/golden/sparse-vdb");
        compare_with_golden_data(vdb_dir, golden_dir)?;

        Ok(())
    }
}
