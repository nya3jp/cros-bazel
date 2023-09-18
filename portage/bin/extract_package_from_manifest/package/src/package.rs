// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::filters::filter_shared_libs;
use anyhow::{Context, Result};
use binarypackage::BinaryPackage;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use version::Version;

pub use common_extract_tarball::TarballContent;

#[derive(Clone, Debug, clap::Args)]
pub struct PackageCommonArgs {
    #[arg(long, help = "Similar to $LD_LIBRARY_PATH, but regexes.")]
    pub shared_library_dir_regex: Vec<Regex>,

    #[arg(long, help = "A regex matching all header files we care about.")]
    pub header_file_dir_regex: Vec<Regex>,
}

/// A unique *stable* identifier for a package.
/// package version is unsuitable here because we don't want uprevs to modify the package id.
#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq, Clone, Debug)]
pub struct PackageUid {
    pub name: String,
    pub slot: String,
}

/// A package, including both analysis-phase metadata accessible to bazel, and runtime metadata
/// like package contents accessible to the actions.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Package {
    #[serde(flatten)]
    pub uid: PackageUid,
    #[serde(flatten)]
    pub tarball_content: TarballContent,

    pub header_files: Vec<PathBuf>,
    pub shared_libraries: Vec<PathBuf>,
}

impl PartialOrd for Package {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Package {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uid.cmp(&other.uid)
    }
}

impl Package {
    pub fn create(
        binpkg_path: &Path,
        out_dir: &Path,
        package_common_args: &PackageCommonArgs,
    ) -> Result<Self> {
        let mut binpkg = BinaryPackage::open(&binpkg_path)
            .with_context(|| format!("Failed to open {binpkg_path:?}"))?;

        let tarball_content =
            common_extract_tarball::extract_tarball(&mut binpkg.archive()?, out_dir, |path| {
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

        let files: Vec<&Path> = tarball_content.all_files().collect();

        let header_files = crate::filters::filter_header_files(
            &files,
            &package_common_args.header_file_dir_regex,
        )?
        .header_files;
        let shared_libraries =
            filter_shared_libs(&files, &package_common_args.shared_library_dir_regex)?
                .iter()
                .map(|p| p.to_path_buf())
                .collect();
        Ok(Package {
            uid: PackageUid {
                name: name.to_string(),
                slot: binpkg.slot().to_string(),
            },
            tarball_content,
            header_files,
            shared_libraries,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn extract_demo_sysroot() -> Result<()> {
        let r = runfiles::Runfiles::create()?;
        for (path, want) in [
            (
                "cros/bazel/portage/common/testdata/shared_libs.tbz2",
                Package {
                    uid: PackageUid {
                        name: "demo/shared_libs".into(),
                        slot: "0/0".into(),
                    },
                    tarball_content: TarballContent {
                        symlinks: vec!["/lib64/libfoo.so".into()],
                        files: vec![
                            "/lib32/libbaz.so.1.2.3".into(),
                            "/lib32/libfoo.so.1.2.3".into(),
                            "/lib64/libbar.so.1.2.3".into(),
                            "/lib64/libfoo.so.1.2.3".into(),
                        ],
                    },
                    header_files: vec![],
                    shared_libraries: vec![
                        "/lib32/libbaz.so.1.2.3".into(),
                        "/lib64/libbar.so.1.2.3".into(),
                        "/lib64/libfoo.so".into(),
                        "/lib64/libfoo.so.1.2.3".into(),
                    ],
                },
            ),
            (
                "cros/bazel/portage/common/testdata/system_headers.tbz2",
                Package {
                    uid: PackageUid {
                        name: "demo/system_headers".into(),
                        slot: "0/0".into(),
                    },
                    tarball_content: TarballContent {
                        symlinks: vec![],
                        files: vec![
                            "/usr/include/foo.h".into(),
                            "/usr/include/subdir/bar.h".into(),
                        ],
                    },
                    header_files: vec![
                        "/usr/include/foo.h".into(),
                        "/usr/include/subdir/bar.h".into(),
                    ],
                    shared_libraries: vec![],
                },
            ),
        ] {
            let out = fileutil::SafeTempDir::new()?;
            let pkg = Package::create(
                &r.rlocation(path),
                out.path(),
                &PackageCommonArgs {
                    shared_library_dir_regex: vec![Regex::new("/lib64")?, Regex::new("/lib32")?],
                    header_file_dir_regex: vec![Regex::new("/usr/include")?],
                },
            )?;
            assert_eq!(pkg, want);
        }
        Ok(())
    }
}
