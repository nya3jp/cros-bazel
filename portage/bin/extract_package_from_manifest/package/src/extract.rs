// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use binarypackage::BinaryPackage;
use std::path::Path;
use version::Version;

use crate::package::PackageUid;
use common_extract_tarball::{extract_tarball, TarballContent};

#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) struct ExtractedPackage {
    pub(crate) uid: PackageUid,
    pub(crate) content: TarballContent,
}

/// Extracts the binpkg to out_dir.
pub(crate) fn extract_binpkg(binpkg_path: &Path, out_dir: &Path) -> Result<ExtractedPackage> {
    let mut binpkg = BinaryPackage::open(binpkg_path)
        .with_context(|| format!("Failed to open {binpkg_path:?}"))?;

    let tarball_content = extract_tarball(&mut binpkg.archive()?, out_dir, |path| {
        // HACK: Rename directories that collide with well-known symlinks.
        // sys-apps/gentoo-functions writes to /lib, but /lib is really a symlink
        // provided by glibc to /lib64.
        let path = match path.strip_prefix("lib") {
            Ok(p) if p != Path::new("") => Path::new("lib64").join(p),
            _ => path.to_path_buf(),
        };
        let path = match path.strip_prefix("usr/lib") {
            Ok(p) if p != Path::new("") => Path::new("usr/lib64").join(p),
            _ => path.to_path_buf(),
        };
        // Special-case .build-id, as it's never needed, and the path
        // changes on every rebuild since it appears to be a hash.
        if path.components().any(|c| c.as_os_str() == ".build-id") {
            Ok(None)
        } else {
            Ok(Some(path))
        }
    })
    .with_context(|| format!("While trying to extract {binpkg_path:?}"))?;

    // Strip the version from packages. This ensures that if I were to uprev
    // category/name-r1 to category/name-r2, then I wouldn't have to update
    // the manifest unless a file changed.
    let (name, _) = Version::from_str_suffix(binpkg.category_pf())?;

    Ok(ExtractedPackage {
        uid: PackageUid {
            name: name.to_string(),
            slot: binpkg.slot().main.to_string(),
        },
        content: tarball_content,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_extract_tarball::TarballFile;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    #[test]
    pub fn extract_demo_sysroot() -> Result<()> {
        let file = |path| TarballFile {
            path: PathBuf::from(path),
            symlink: None,
        };
        let symlink = |path, dest| TarballFile {
            path: PathBuf::from(path),
            symlink: Some(PathBuf::from(dest)),
        };

        let r = runfiles::Runfiles::create()?;
        for (path, want) in [
            (
                "cros/bazel/portage/common/testdata/shared_libs.tbz2",
                ExtractedPackage {
                    uid: PackageUid {
                        name: "demo/shared_libs".into(),
                        slot: "0".into(),
                    },
                    content: TarballContent {
                        files: BTreeSet::from([
                            symlink("/lib64/libfoo.so", "/lib64/libfoo.so.1.2.3"),
                            file("/lib32/libbaz.so.1.2.3"),
                            file("/lib32/libfoo.so.1.2.3"),
                            file("/lib64/libbar.so.1.2.3"),
                            file("/lib64/libfoo.so.1.2.3"),
                        ]),
                    },
                },
            ),
            (
                "cros/bazel/portage/common/testdata/system_headers.tbz2",
                ExtractedPackage {
                    uid: PackageUid {
                        name: "demo/system_headers".into(),
                        slot: "0".into(),
                    },
                    content: TarballContent {
                        files: BTreeSet::from([
                            file("/usr/include/foo.h"),
                            file("/usr/include/subdir/bar.h"),
                        ]),
                    },
                },
            ),
        ] {
            let out = fileutil::SafeTempDir::new()?;
            let pkg = extract_binpkg(&runfiles::rlocation!(r, path), out.path())?;
            assert_eq!(pkg, want);
        }
        Ok(())
    }
}
