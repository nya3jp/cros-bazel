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
use crate::package::{FileMetadata, FileType, Package, PackageUid, SymlinkMetadata};

pub struct PackageSet {
    dir: PathBuf,
    packages: Vec<PackageUid>,
    owners: HashMap<PathBuf, (PackageUid, FileType)>,
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
) -> Result<HashMap<PathBuf, (PackageUid, FileType)>> {
    let mut owners: HashMap<PathBuf, (PackageUid, FileType)> = HashMap::new();
    for pkg in packages {
        for file in &pkg.content.files {
            let file_type = match &file.symlink {
                None => FileType::Unknown,
                Some(path) => FileType::Symlink(SymlinkMetadata {
                    target: path.to_path_buf(),
                }),
            };
            if let Some(old_owner) = owners.insert(file.path.clone(), (pkg.uid.clone(), file_type))
            {
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
            .map(|tbz2| extract_binpkg(tbz2.as_ref(), dir))
            .collect::<Result<Vec<ExtractedPackage>>>()?;

        validate_unique_packages(&packages)?;

        Ok(PackageSet {
            dir: dir.to_path_buf(),
            owners: calculate_owners(&packages)?,
            packages: packages.into_iter().map(|pkg| pkg.uid).collect(),
        })
    }

    /// Returns an iterator through all files that we haven't yet determined the file type for.
    fn unknown_files(&self) -> impl Iterator<Item = &Path> {
        self.owners
            .iter()
            .flat_map(|(path, (_, file_type))| match file_type {
                FileType::Unknown => Some(path.as_path()),
                _ => None,
            })
    }

    /// Sets the filetype of a file.
    fn set_filetype(&mut self, path: &Path, value: FileType) -> Result<()> {
        let (_, file_type) = self
            .owners
            .get_mut(path)
            .with_context(|| format!("{path:?} must exist in package set"))?;
        *file_type = value;
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

    pub fn wrap_elf_files(&mut self, ld_library_path: &[PathBuf]) -> Result<()> {
        let metadata =
            crate::elf::wrap_elf_files(&self.dir, ld_library_path, self.unknown_files())?;
        for (path, metadata) in metadata {
            self.set_filetype(&path, FileType::ElfBinary(metadata))?;
        }
        Ok(())
    }

    pub fn into_packages(self) -> Vec<Package> {
        let mut uid_to_files: BTreeMap<PackageUid, BTreeMap<PathBuf, FileMetadata>> =
            BTreeMap::new();
        for (path, (pkg, file_type)) in self.owners {
            uid_to_files
                .entry(pkg)
                .or_default()
                .insert(path, FileMetadata { file_type });
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
    use crate::elf::ElfFileMetadata;
    use anyhow::Context;
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
            HashMap::from([(PathBuf::from("/foo"), (foo.uid.clone(), FileType::Unknown))])
        );

        assert_eq!(
            calculate_owners(&[foo.clone(), bar.clone()])?,
            HashMap::from([
                (PathBuf::from("/foo"), (foo.uid.clone(), FileType::Unknown)),
                (PathBuf::from("/bar"), (bar.uid.clone(), FileType::Unknown)),
            ])
        );

        assert!(calculate_owners(&[foo, dup_foo]).is_err());
        Ok(())
    }

    #[test]
    fn package_set_cuj() -> Result<()> {
        let default_elf = ElfFileMetadata {
            interp: "/lib64/ld-linux-x86-64.so.2".into(),
            libs: Default::default(),
            rpath: Default::default(),
            runpath: Default::default(),
        };

        let r = runfiles::Runfiles::create()?;

        let glibc_tbz2 = runfiles::rlocation!(r, "files/testdata_glibc");
        let ncurses_tbz2 = runfiles::rlocation!(r, "files/testdata_ncurses");
        let nano_tbz2 = runfiles::rlocation!(r, "files/testdata_nano");

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
            package_set.wrap_elf_files(&ld_library_path)?;

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
                    slot: "0".into(),
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
                file_type: FileType::SharedLibrary
            })
        );
        assert_eq!(
            glibc.content.get(Path::new("/lib64/ld-linux.so.2")),
            Some(&FileMetadata {
                file_type: FileType::Symlink(SymlinkMetadata {
                    target: PathBuf::from("/lib32/ld-linux.so.2")
                })
            })
        );

        // While this is a shared library, it shouldn't count because it's not in LD_LIBRARY_PATH.
        assert_eq!(
            glibc.content.get(Path::new("/usr/lib64/gconv/IBM278.so")),
            Some(&FileMetadata {
                file_type: FileType::Unknown
            })
        );

        assert_eq!(
            ncurses.content.get(Path::new("/usr/include/curses.h")),
            Some(&FileMetadata {
                file_type: FileType::HeaderFile
            })
        );

        assert_eq!(
            glibc.content.get(Path::new("/usr/bin/locale")),
            Some(&FileMetadata {
                file_type: FileType::ElfBinary(ElfFileMetadata {
                    libs: [("libc.so.6".to_string(), PathBuf::from("/lib64/libc.so.6"))].into(),
                    ..default_elf.clone()
                })
            })
        );

        assert_eq!(
            ncurses.content.get(Path::new("/usr/include/ncurses.h")),
            Some(&FileMetadata {
                file_type: FileType::Symlink(SymlinkMetadata {
                    target: PathBuf::from("/usr/include/curses.h")
                })
            })
        );

        assert_eq!(
            nano.content.get(Path::new("/bin/nano")),
            Some(&FileMetadata {
                file_type: FileType::ElfBinary(ElfFileMetadata {
                    libs: [
                        ("libc.so.6".to_string(), PathBuf::from("/lib64/libc.so.6")),
                        (
                            "libncursesw.so.6".to_string(),
                            PathBuf::from("/usr/lib64/libncursesw.so.6")
                        ),
                        (
                            "libtinfow.so.6".to_string(),
                            PathBuf::from("/usr/lib64/libtinfow.so.6")
                        ),
                    ]
                    .into(),
                    ..default_elf
                })
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
            package_set.wrap_elf_files(&ld_library_path)?;
            let got = &package_set.into_packages()[0];
            assert_eq!(*got, want)
        }

        assert!(out.join("bin/nano").is_file());
        assert!(out.join("bin/nano.elf").is_file());
        assert!(out.join("bin/rnano").is_file());
        assert!(!out.join("bin/rnano.elf").is_file());

        Ok(())
    }
}
