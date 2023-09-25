// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    iter::Iterator,
    path::{Path, PathBuf},
};

use crate::extract::{extract_binpkg, ExtractedPackage};
use crate::package::{FileMetadata, FileType, Package, PackageUid};

pub struct PackageSet {
    packages: Vec<PackageUid>,
    owners: HashMap<PathBuf, (PackageUid, FileMetadata)>,
}

/// Validates that no two packages have the same unique identifier.
fn validate_unique_packages(packages: &[ExtractedPackage]) -> Result<()> {
    let mut unique_packages: BTreeSet<&PackageUid> = BTreeSet::new();
    for pkg in packages {
        if !unique_packages.insert(&pkg.uid) {
            bail!("Found multiple tbz2 files for package {:?}", pkg.uid)
        }
    }
    Ok(())
}

/// Creates a mapping from file to the package which it came from.
/// If multiple packages contain the same file, then returns an error.
fn calculate_owners(
    packages: &[ExtractedPackage],
) -> Result<HashMap<PathBuf, (PackageUid, FileMetadata)>> {
    let mut owners: HashMap<PathBuf, (PackageUid, FileMetadata)> = HashMap::new();
    for pkg in packages {
        for file in &pkg.content.files {
            if let Some(old_owner) = owners.insert(
                file.path.clone(),
                (
                    pkg.uid.clone(),
                    FileMetadata {
                        symlink: file.symlink.is_some(),
                        kind: FileType::Unknown,
                    },
                ),
            ) {
                bail!(
                    "Conflict: Packages {:?} and {:?} both create file {:?}",
                    old_owner,
                    pkg.uid,
                    file.path,
                );
            }
        }
    }
    Ok(owners)
}

impl PackageSet {
    /// Creates a set of packages, validating that:
    /// * No two packages have the same identifier.
    /// * No two packages contain the same file
    pub fn create<P: AsRef<Path>>(dir: &Path, packages: &[P]) -> Result<Self> {
        let packages = packages
            .iter()
            .map(|tbz2| extract_binpkg(tbz2.as_ref(), &dir))
            .collect::<Result<Vec<ExtractedPackage>>>()?;

        validate_unique_packages(&packages)?;

        Ok(PackageSet {
            owners: calculate_owners(&packages)?,
            packages: packages.into_iter().map(|pkg| pkg.uid).collect(),
        })
    }

    /// Returns an iterator through all files that we haven't yet determined the file type for.
    fn unknown_files(&self) -> impl Iterator<Item = &Path> {
        self.owners.iter().flat_map(|(path, (_, metadata))| {
            if metadata.kind == FileType::Unknown {
                Some(path.as_path())
            } else {
                None
            }
        })
    }

    /// Sets the filetype of a file.
    fn set_filetype(&mut self, path: &Path, value: FileType) -> Result<()> {
        let (_, metadata) = self
            .owners
            .get_mut(path)
            .with_context(|| format!("{path:?} must exist in package set"))?;
        metadata.kind = value;
        Ok(())
    }

    pub fn fill_headers(&mut self, header_file_dir_regexes: &[Regex]) -> Result<BTreeSet<PathBuf>> {
        let header_files = crate::headers::filter_header_files(
            &self.unknown_files().collect::<Vec<_>>(),
            header_file_dir_regexes,
        )?;
        for path in header_files.header_files {
            self.set_filetype(&path, FileType::HeaderFile)?;
        }
        Ok(header_files.header_file_dirs)
    }

    pub fn generate_ld_library_path(&self, directory_regexes: &[Regex]) -> Result<Vec<PathBuf>> {
        crate::library_path::generate_ld_library_path(
            &self.unknown_files().collect::<Vec<_>>(),
            directory_regexes,
        )
    }

    pub fn fill_shared_libraries(&mut self, ld_library_path: &[PathBuf]) -> Result<()> {
        let shared_libs = crate::library_path::calculate_shared_libraries(
            &self.unknown_files().collect::<Vec<_>>(),
            ld_library_path,
        )?;
        // Convert it from a Path that borrows self immutably to a PathBuf to allow us to call
        // set_filetype safely.
        let shared_libs = shared_libs
            .iter()
            .map(|p| p.to_path_buf())
            .collect::<Vec<_>>();
        for path in shared_libs {
            self.set_filetype(&path, FileType::SharedLibrary)?;
        }
        Ok(())
    }

    pub fn into_packages(self) -> Vec<Package> {
        let mut uid_to_files: BTreeMap<PackageUid, BTreeMap<PathBuf, FileMetadata>> =
            BTreeMap::new();
        for (path, (pkg, metadata)) in self.owners {
            uid_to_files.entry(pkg).or_default().insert(path, metadata);
        }
        // Preserve the original package order.
        self.packages
            .into_iter()
            .map(|uid| Package {
                content: uid_to_files.entry(uid.clone()).or_default().clone(),
                uid,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_extract_tarball::{TarballContent, TarballFile};

    fn gen_package(name: &str, content: &[&Path]) -> ExtractedPackage {
        ExtractedPackage {
            uid: PackageUid {
                name: name.to_string(),
                slot: "0/0".to_string(),
            },
            content: TarballContent {
                files: content
                    .iter()
                    .map(|p| TarballFile {
                        path: p.to_path_buf(),
                        symlink: None,
                    })
                    .collect(),
            },
        }
    }

    fn get_package(packages: &[Package], uid: &PackageUid) -> Result<Package> {
        packages
            .iter()
            .find(|p| p.uid == *uid)
            .cloned()
            .with_context(|| "Unable to find package {uid:?}")
    }

    #[test]
    fn duplicate_packages() -> Result<()> {
        validate_unique_packages(&[gen_package("a", &[]), gen_package("b", &[])])?;
        assert!(validate_unique_packages(&[gen_package("a", &[]), gen_package("a", &[])]).is_err());
        Ok(())
    }

    #[test]
    fn duplicate_files() -> Result<()> {
        let foo = gen_package("foo", &[Path::new("/foo")]);
        let bar = gen_package("bar", &[Path::new("/bar")]);
        let dup_foo = gen_package("dup_foo", &[Path::new("/foo")]);

        assert_eq!(
            calculate_owners(&[foo.clone()])?,
            HashMap::from([(
                PathBuf::from("/foo"),
                (foo.uid.clone(), FileMetadata::new_file())
            )])
        );

        assert_eq!(
            calculate_owners(&[foo.clone(), bar.clone()])?,
            HashMap::from([
                (
                    PathBuf::from("/foo"),
                    (foo.uid.clone(), FileMetadata::new_file())
                ),
                (
                    PathBuf::from("/bar"),
                    (bar.uid.clone(), FileMetadata::new_file())
                ),
            ])
        );

        assert!(calculate_owners(&[foo, dup_foo]).is_err());
        Ok(())
    }

    #[test]
    fn package_set_cuj() -> Result<()> {
        let r = runfiles::Runfiles::create()?;

        let glibc_tbz2 = r.rlocation("files/testdata_glibc");
        let ncurses_tbz2 = r.rlocation("files/testdata_ncurses");
        let nano_tbz2 = r.rlocation("files/testdata_nano");

        let request_header_regexes = vec![Regex::new("/usr/include")?];
        let want_headers = BTreeSet::from([PathBuf::from("/usr/include")]);
        let ld_library_path = vec![PathBuf::from("/lib64"), PathBuf::from("/usr/lib64")];

        // Simulate the workflow for a real package.
        // Step 1: run update_manifest.
        let (glibc, ncurses, nano) = {
            let out = fileutil::SafeTempDir::new()?;
            let out = out.path();
            // Construct it in an arbitrary order. The order shouldn't matter.
            let mut package_set = PackageSet::create(
                out,
                &[
                    nano_tbz2.as_path(),
                    ncurses_tbz2.as_path(),
                    glibc_tbz2.as_path(),
                ],
            )?;

            let headers = package_set.fill_headers(&request_header_regexes)?;
            assert_eq!(headers, want_headers);

            let got_ld_library_path = package_set
                .generate_ld_library_path(&[Regex::new("/lib64")?, Regex::new("/usr/lib64")?])?;
            assert_eq!(got_ld_library_path, ld_library_path);
            package_set.fill_shared_libraries(&ld_library_path)?;

            let packages = package_set.into_packages();

            let glibc = get_package(
                &packages,
                &PackageUid {
                    name: "sys-libs/glibc".into(),
                    slot: "2.2".into(),
                },
            )?;

            let ncurses = get_package(
                &packages,
                &PackageUid {
                    name: "sys-libs/ncurses".into(),
                    slot: "0/6".into(),
                },
            )?;

            let nano = get_package(
                &packages,
                &PackageUid {
                    name: "app-editors/nano".into(),
                    slot: "0".into(),
                },
            )?;

            (glibc, ncurses, nano)
        };

        // Verify that the contents of the packages are correct.
        assert_eq!(
            glibc.content.get(Path::new("/lib64/ld-linux-x86-64.so.2")),
            Some(&FileMetadata {
                symlink: false,
                kind: FileType::SharedLibrary
            })
        );
        assert_eq!(
            glibc.content.get(Path::new("/lib64/ld-linux.so.2")),
            Some(&FileMetadata {
                symlink: true,
                kind: FileType::SharedLibrary
            })
        );

        // While this is a shared library, it shouldn't count because it's not in LD_LIBRARY_PATH.
        assert_eq!(
            glibc.content.get(Path::new("/usr/lib64/gconv/IBM278.so")),
            Some(&FileMetadata {
                symlink: false,
                kind: FileType::Unknown
            })
        );

        assert_eq!(
            ncurses.content.get(Path::new("/usr/include/ncurses.h")),
            Some(&FileMetadata {
                symlink: true,
                kind: FileType::HeaderFile,
            })
        );

        // Step 2: Extract packages. Here, each package is its own action, so each package gets
        // their own package set. However, each package has its transitive dependencies already
        // installed into the directory.
        let out = fileutil::SafeTempDir::new()?;
        let out = out.path();
        for (tbz2, want) in [
            (glibc_tbz2, glibc),
            (ncurses_tbz2, ncurses),
            (nano_tbz2, nano),
        ] {
            let mut package_set = PackageSet::create(out, &[tbz2.as_path()])?;

            package_set.fill_headers(&request_header_regexes)?;
            package_set.fill_shared_libraries(&ld_library_path)?;
            let got = &package_set.into_packages()[0];
            assert_eq!(*got, want)
        }

        assert!(out.join("bin/nano").is_file());
        assert!(out.join("bin/rnano").is_file());

        Ok(())
    }
}
