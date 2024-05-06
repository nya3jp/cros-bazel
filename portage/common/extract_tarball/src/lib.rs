// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{anyhow, bail, Context, Ok, Result};
use path_absolutize::Absolutize;
use std::{
    collections::{BTreeSet, HashMap},
    io::Read,
    path::{Path, PathBuf},
};
use tar::EntryType;

#[derive(Clone, Default, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct TarballFile {
    pub path: PathBuf,
    // If the path is a symlink, this is the absolute path of the target.
    // eg. If /path/to/foo/bar is a link to "../baz", this would be "/path/to/baz".
    pub symlink: Option<PathBuf>,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct TarballContent {
    pub files: BTreeSet<TarballFile>,
}

/// Resolves a symlink transitively until it reaches a non-symlink.
fn resolve_symlink<'a>(symlinks: &'a HashMap<PathBuf, PathBuf>, dest: &'a Path) -> &'a Path {
    match symlinks.get(dest) {
        Some(transitive_dest) => resolve_symlink(symlinks, transitive_dest),
        None => dest,
    }
}

/// Extracts from the specified tarball to out_dir.
/// If out_dir is specified, then we extract to the specified directory.
/// want_file is a mapping from the path within the archive to the path outside, relative to
/// out_dir.
/// For example:
/// * If we wanted to extract lib/foo to ${out_dir}/lib64/foo, we would map lib/foo to lib64/foo.
/// * If we wanted to not extract build_id, we would map path/to/.build_id/... to None.
/// Returns the contents of out_dir (or what it would have contained)
pub fn extract_tarball(
    archive: &mut tar::Archive<impl Sized + Read>,
    out_dir: &Path,
    want_file: impl Fn(&Path) -> Result<Option<PathBuf>>,
) -> Result<TarballContent> {
    let mut out_files: Vec<TarballFile> = vec![];
    let mut path_mapping: HashMap<PathBuf, PathBuf> = HashMap::new();
    for entry_result in archive.entries()? {
        let mut entry = entry_result?;
        let header = &entry.header();
        let path = entry.path()?;
        let path = path.strip_prefix(".")?.to_path_buf();
        if let Some(relative_path) = want_file(&path)? {
            let absolute_path = Path::new("/").join(&relative_path);
            path_mapping.insert(Path::new("/").join(&path), absolute_path.clone());
            let out_path = out_dir.join(&relative_path);
            let dir = out_path.parent().context("Path must have parent")?;
            std::fs::create_dir_all(dir).with_context(|| "Failed to create {dir:?}")?;
            match header.entry_type() {
                EntryType::Directory => {
                    // Directories can't be extracted in a meaningful manner, because bazel
                    // doesn't deal with directories. It can only deal with individual files.
                    continue;
                }
                EntryType::Regular => {
                    entry.unpack(out_path).context("Failed to extract file")?;
                    out_files.push(TarballFile {
                        path: absolute_path,
                        symlink: None,
                    });
                }
                // Treat hard links the same as symlinks for now. We'll change this if this
                // becomes a problem.
                EntryType::Symlink | EntryType::Link => {
                    let dest = header
                        .link_name()?
                        .ok_or_else(|| anyhow!("Link name doesn't exist"))?;
                    let dest = if header.entry_type().is_hard_link() {
                        // Since the archive always has files starting with "./", hard links will
                        // always start with "./".
                        Path::new("/").join(dest.strip_prefix(".")?)
                    } else if dest.is_absolute() {
                        dest.to_path_buf()
                    } else {
                        Path::new("/").join(path.parent().unwrap().join(dest))
                    };
                    out_files.push(TarballFile {
                        path: absolute_path,
                        symlink: Some(dest.absolutize()?.to_path_buf()),
                    });
                }
                entry_type => bail!("Unsupported tar entry type: {:?}", entry_type),
            }
        }
    }

    let mut symlinks: HashMap<PathBuf, PathBuf> = HashMap::new();
    for file in &mut out_files {
        if let Some(symlink) = file.symlink.as_mut() {
            *symlink = path_mapping
                .get_mut(symlink.as_path())
                .with_context(|| {
                    format!(
                        "{:?} is a symlink to {:?}, which doesn't exist in the archive",
                        file.path, symlink,
                    )
                })?
                .clone();
            symlinks.insert(file.path.to_path_buf(), symlink.to_path_buf());
        }
    }

    let out_files: BTreeSet<TarballFile> = out_files
        .into_iter()
        .map(|f| TarballFile {
            path: f.path,
            symlink: f
                .symlink
                .map(|dst| resolve_symlink(&symlinks, &dst).to_path_buf()),
        })
        .collect();

    for file in &out_files {
        if let Some(symlink) = &file.symlink {
            let target = pathdiff::diff_paths(symlink, file.path.parent().unwrap())
                .context("Unable to diff paths")?;
            let path = out_dir.join(file.path.strip_prefix(Path::new("/"))?);
            std::os::unix::fs::symlink(&target, &path)
                .with_context(|| format!("Failed to symlink {path:?} to {target:?}"))?;
        }
    }

    Ok(TarballContent { files: out_files })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::os::unix::prelude::MetadataExt;

    use binarypackage::BinaryPackage;
    use fileutil::SafeTempDir;

    use super::*;

    const NANORC_SIZE: u64 = 11225;
    const NANO_SIZE: u64 = 274784;

    const BINARY_PKG_RUNFILE: &str = "files/testdata_nano";

    fn binary_package() -> Result<BinaryPackage> {
        let r = runfiles::Runfiles::create()?;
        BinaryPackage::open(&runfiles::rlocation!(r, BINARY_PKG_RUNFILE))
    }

    #[test]
    fn transitive_symlinks() {
        let symlinks: HashMap<PathBuf, PathBuf> = HashMap::from([
            (
                PathBuf::from("/lib64/libfoo.so.1"),
                PathBuf::from("/lib64/libfoo.so.1.2"),
            ),
            (
                PathBuf::from("/lib64/libfoo.so.1.2"),
                PathBuf::from("/lib64/libfoo.so.1.2.3"),
            ),
        ]);
        assert_eq!(
            resolve_symlink(&symlinks, Path::new("/lib64/libfoo.so.1.2.3")),
            Path::new("/lib64/libfoo.so.1.2.3")
        );
        assert_eq!(
            resolve_symlink(&symlinks, Path::new("/lib64/libfoo.so.1.2")),
            Path::new("/lib64/libfoo.so.1.2.3")
        );
        assert_eq!(
            resolve_symlink(&symlinks, Path::new("/lib64/libfoo.so.1")),
            Path::new("/lib64/libfoo.so.1.2.3")
        );
    }

    #[test]
    fn extracts_out_files() -> Result<()> {
        let tmp_dir = SafeTempDir::new()?;
        let mut bp = binary_package()?;
        let mapping = [
            (PathBuf::from("/bin/nano"), PathBuf::from("nano")),
            (PathBuf::from("/etc/nanorc"), PathBuf::from("nanorc")),
            (PathBuf::from("/bin/rnano"), PathBuf::from("foo/rnano")),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        let content = extract_tarball(&mut bp.archive()?, tmp_dir.path(), |path| {
            Ok(mapping.get(&Path::new("/").join(path)).cloned())
        })?;

        assert_eq!(
            content,
            TarballContent {
                files: [
                    TarballFile {
                        path: PathBuf::from("/nano"),
                        symlink: None
                    },
                    TarballFile {
                        path: PathBuf::from("/nanorc"),
                        symlink: None
                    },
                    TarballFile {
                        path: PathBuf::from("/foo/rnano"),
                        symlink: Some("/nano".into())
                    },
                ]
                .into(),
            }
        );

        let nano_md = std::fs::metadata(tmp_dir.path().join("nano"))?;
        assert_eq!(nano_md.mode() & 0o777, 0o755);
        assert_eq!(nano_md.size(), NANO_SIZE);

        let nanorc_md = std::fs::metadata(tmp_dir.path().join("nanorc"))?;
        assert_eq!(nanorc_md.mode() & 0o777, 0o644);
        assert_eq!(nanorc_md.size(), NANORC_SIZE);

        let rnano_path = tmp_dir.path().join("foo/rnano");
        let rnano_md = std::fs::symlink_metadata(&rnano_path)?;
        assert!(rnano_md.is_symlink());
        assert_eq!(std::fs::read_link(rnano_path)?, Path::new("../nano"));
        assert_eq!(rnano_md.mode() & 0o777, 0o777);

        Ok(())
    }
}
