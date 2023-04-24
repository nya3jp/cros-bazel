// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Context;
use anyhow::{anyhow, bail, Error, Result};
use itertools::Itertools;
use once_cell::sync::Lazy;
use rayon::prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use regex::Regex;
use sha2::digest::generic_array::GenericArray;
use std::fs::{read_link, File};
use std::io;
use std::os::unix::prelude::OsStrExt;
use std::{
    borrow::Borrow,
    collections::{HashMap, VecDeque},
    ffi::OsStr,
    fs::read_to_string,
    io::ErrorKind,
    iter,
    path::{Path, PathBuf},
};

use walkdir::{DirEntry, WalkDir};

use sha2::{Digest, Sha256};

use topological_sort::TopologicalSort;

use crate::{
    config::{bundle::ConfigBundle, site::SiteSettings, ConfigSource},
    data::Vars,
};

pub type Sha256Digest = GenericArray<u8, sha2::digest::consts::U32>;

/// A regular expression matching a line of metadata/layout.conf.
static LAYOUT_CONF_LINE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(\S+)\s*=\s*(.*)$").unwrap());

/// Layout information of a Portage repository that is loaded from `metadata/layout.conf`.
///
/// This struct is used in the middle of loading repositories in [`RepositorySet`].
#[derive(Debug)]
struct RepositoryLayout {
    name: String,
    base_dir: PathBuf,
    parents: Vec<String>,
}

impl RepositoryLayout {
    /// Loads `metadata/layout.conf` from a directory.
    fn load(base_dir: &Path) -> Result<Self> {
        let path = base_dir.join("metadata/layout.conf");
        let context = || format!("Failed to load {}", path.display());

        let content = read_to_string(&path).with_context(context)?;

        let mut name: Option<String> = None;
        let mut parents = Vec::<String>::new();

        for (lineno, line) in content.split('\n').enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let caps = LAYOUT_CONF_LINE_RE
                .captures(line)
                .ok_or_else(|| anyhow!("Line {}: syntax error", lineno + 1))
                .with_context(context)?;
            let key = caps.get(1).unwrap().as_str();
            let value = caps.get(2).unwrap().as_str();
            match key {
                "repo-name" => {
                    name = Some(value.to_owned());
                }
                "masters" => {
                    parents = value
                        .split_ascii_whitespace()
                        .map(|s| s.to_owned())
                        .collect();
                }
                _ => {
                    // Ignore unsupported entries.
                }
            }
        }

        let name = name
            .ok_or_else(|| anyhow!("repo-name not defined"))
            .with_context(context)?;
        Ok(Self {
            name,
            base_dir: base_dir.to_owned(),
            parents,
        })
    }
}

/// A map of RepositoryLayout, keyed by repository names.
///
/// This map is used in the middle of loading repositories in [`RepositorySet`].
type RepositoryLayoutMap = HashMap<String, RepositoryLayout>;

/// Holds [`PathBuf`] of various file paths related to a repository.
///
/// This is used to implement [`Repository`]'s getters.
#[derive(Clone, Debug)]
struct RepositoryLocation {
    base_dir: PathBuf,
    eclass_dir: PathBuf,
    profiles_dir: PathBuf,
}

impl RepositoryLocation {
    fn new(base_dir: &Path) -> Self {
        Self {
            base_dir: base_dir.to_owned(),
            eclass_dir: base_dir.join("eclass"),
            profiles_dir: base_dir.join("profiles"),
        }
    }
}

/// Represents a Portage repository (aka "overlay").
#[derive(Clone, Debug)]
pub struct Repository {
    name: String,
    location: RepositoryLocation,
    parents: Vec<RepositoryLocation>,
}

impl Repository {
    /// Creates a new [`Repository`] from a repository name and [`RepositoryLayoutMap`].
    fn new(name: &str, layout_map: &RepositoryLayoutMap) -> Result<Self> {
        let layout = layout_map
            .get(name)
            .ok_or_else(|| anyhow!("repository {} not found", name))?;
        let location = RepositoryLocation::new(&layout.base_dir);
        let parents = layout
            .parents
            .iter()
            .map(|name| {
                layout_map
                    .get(name)
                    .map(|layout| RepositoryLocation::new(&layout.base_dir))
                    .ok_or_else(|| anyhow!("repository {} not found", name))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            name: name.to_owned(),
            location,
            parents,
        })
    }

    /// Creates a repository with no parents. This is useful for unit testing.
    pub fn new_simple(name: &str, path: &Path) -> Self {
        Self {
            name: name.to_string(),
            location: RepositoryLocation::new(path),
            parents: vec![],
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn base_dir(&self) -> &Path {
        &self.location.base_dir
    }

    pub fn eclass_dirs(&self) -> impl Iterator<Item = &Path> {
        iter::once(self.location.eclass_dir.borrow()).chain(
            self.parents
                .iter()
                .map(|location| location.eclass_dir.borrow()),
        )
    }

    pub fn profiles_dir(&self) -> &Path {
        &self.location.profiles_dir
    }

    /// Scans the repository and returns ebuild file paths for the specified
    /// package.
    pub fn find_ebuilds(&self, package_name: &str) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::<PathBuf>::new();
        match self.location.base_dir.join(package_name).read_dir() {
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    Ok(Vec::new())
                } else {
                    Err(Error::new(err))
                }
            }
            Ok(read_dir) => {
                for entry in read_dir {
                    let path = entry?.path();
                    // TODO: Consider filtering by file name stems.
                    if path.extension() == Some(OsStr::new("ebuild")) {
                        paths.push(path);
                    }
                }
                Ok(paths)
            }
        }
    }

    /// Scans the repository and returns all ebuild file paths.
    pub fn find_all_ebuilds(&self) -> Result<Vec<PathBuf>> {
        let mut ebuild_paths = Vec::<PathBuf>::new();

        // Find */*/*.ebuild.
        // TODO: Consider categories listed in `profiles/categories`.
        for category_entry in self.location.base_dir.read_dir()? {
            let category_path = category_entry?.path();
            if !category_path.is_dir() {
                continue;
            }
            for package_entry in category_path.read_dir()? {
                let package_path = package_entry?.path();
                if !package_path.is_dir() {
                    continue;
                }
                for ebuild_entry in package_path.read_dir()? {
                    let ebuild_path = ebuild_entry?.path();
                    // TODO: Consider filtering by file name stems.
                    if ebuild_path.extension() == Some(OsStr::new("ebuild")) {
                        ebuild_paths.push(ebuild_path);
                    }
                }
            }
        }

        // Make the order deterministic.
        ebuild_paths.sort();

        Ok(ebuild_paths)
    }
}

/// Holds a set of at least one [`Repository`].
#[derive(Clone, Debug)]
pub struct RepositorySet {
    root_dir: PathBuf,
    repos: HashMap<String, Repository>,
    // Keeps the insertion order of `repos`.
    order: Vec<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RepositoryDigest {
    pub file_hashes: Vec<(PathBuf, Sha256Digest)>,
    pub repo_hash: Sha256Digest,
}

impl RepositorySet {
    /// Loads repositories configured for a configuration root directory.
    ///
    /// It evaluates `make.conf` in configuration directories tunder `root_dir`
    /// to locate the primary repository (from `$PORTDIR`) and secondary
    /// repositories (from `$PORTDIR_OVERLAY`), and then loads those
    /// repositories.
    pub fn load(root_dir: &Path) -> Result<Self> {
        // Locate repositories by reading PORTDIR and PORTDIR_OVERLAY in make.conf.
        let site_settings = SiteSettings::load(root_dir)?;
        let bootstrap_config = {
            let mut env = Vars::new();
            let nodes = site_settings.evaluate_configs(&mut env);
            ConfigBundle::new(env, nodes)
        };

        let primary_repo_dir = bootstrap_config
            .env()
            .get("PORTDIR")
            .cloned()
            .ok_or_else(|| anyhow!("PORTDIR is not defined in system configs"))?;
        let secondary_repo_dirs = bootstrap_config
            .env()
            .get("PORTDIR_OVERLAY")
            .cloned()
            .unwrap_or_default();

        // Read layout.conf in repositories to build a map from repository names
        // to repository layout info.
        let mut layout_map = HashMap::<String, RepositoryLayout>::new();
        let mut order: Vec<String> = Vec::new();
        for repo_dir in iter::once(primary_repo_dir.borrow())
            .chain(secondary_repo_dirs.split_ascii_whitespace())
        {
            let repo_dir = PathBuf::from(repo_dir);
            // TODO(b/264959615): Delete this once crossdev is deleted from
            // the PORTDIR_OVERLAY.
            if repo_dir == PathBuf::from("/usr/local/portage/crossdev") {
                eprintln!("Skipping crossdev repo");
                continue;
            }

            let layout = RepositoryLayout::load(&repo_dir)?;
            let name = layout.name.to_owned();
            if let Some(old_layout) = layout_map.insert(name.to_owned(), layout) {
                bail!(
                    "multiple repositories have the same name: {}",
                    old_layout.name
                );
            }
            order.push(name);
        }

        if order.is_empty() {
            bail!("Repository contains no overlays");
        }

        // Finally, build a map from repository names to Repository objects,
        // resolving references.
        let repos: HashMap<String, Repository> = layout_map
            .keys()
            .map(|name| Repository::new(name, &layout_map))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|repo| (repo.name().to_owned(), repo))
            .collect();

        Ok(Self {
            root_dir: root_dir.to_owned(),
            repos,
            order,
        })
    }

    /// Returns the repositories from most generic to most specific.
    pub fn get_repos(&self) -> Vec<&Repository> {
        let mut repo_list: Vec<&Repository> = Vec::new();
        for name in &self.order {
            repo_list.push(self.get_repo_by_name(name).unwrap());
        }

        repo_list
    }

    /// Returns the primary/leaf repository.
    ///
    /// i.e., overlay-arm64-generic
    pub fn primary(&self) -> &Repository {
        let name = self
            .order
            .last()
            .expect("repository set should not be empty");
        self.get_repo_by_name(name).unwrap()
    }

    /// Looks up a repository by its name.
    pub fn get_repo_by_name(&self, name: &str) -> Result<&Repository> {
        self.repos
            .get(name)
            .ok_or_else(|| anyhow!("repository not found: {}", name))
    }

    /// Looks up a repository that contains the specified file path.
    /// It can be used, for example, to look up a repository that contains an
    /// ebuild file.
    pub fn get_repo_by_path<'a, 'b>(&'a self, dir: &'b Path) -> Result<(&'a Repository, &'b Path)> {
        if !dir.is_absolute() {
            bail!(
                "BUG: absolute path required to lookup repositories: {}",
                dir.to_string_lossy()
            );
        }
        for repo in self.repos.values() {
            if let Ok(rel_path) = dir.strip_prefix(repo.base_dir()) {
                return Ok((repo, rel_path));
            }
        }
        bail!("repository not found under {}", dir.to_string_lossy());
    }

    /// Scans the repositories and returns ebuild file paths for the specified
    /// package.
    ///
    /// When there are two or more repositories, returned ebuild paths are
    /// sorted so that one from a lower-priority repository comes before one
    /// from a higher-priority repository.
    pub fn find_ebuilds(&self, package_name: &str) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::<PathBuf>::new();
        for repo in self.get_repos() {
            paths.extend(repo.find_ebuilds(package_name)?);
        }
        Ok(paths)
    }

    /// Scans the repositories and returns all ebuild file paths.
    ///
    /// When there are two or more repositories, returned ebuild paths are
    /// sorted so that one from a lower-priority repository comes before one
    /// from a higher-priority repository.
    pub fn find_all_ebuilds(&self) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::<PathBuf>::new();
        for repo in self.get_repos() {
            paths.extend(repo.find_all_ebuilds()?);
        }
        Ok(paths)
    }

    /// Filters .git, files directories, etc
    fn ignore_filter(entry: &DirEntry) -> bool {
        entry.file_name() == ".git"
            || (entry.file_type().is_dir() && entry.file_name() == "md5-cache")
    }

    fn hash_file(path: PathBuf) -> Result<(PathBuf, Sha256Digest)> {
        let mut file = File::open(&path).context("Failed to open {path:?}")?;
        let mut hasher = Sha256::new();
        io::copy(&mut file, &mut hasher).context("Failed to read {path:?}")?;
        let hash = hasher.finalize();

        Ok((path, hash))
    }

    fn hash_symlink(path: PathBuf) -> Result<(PathBuf, Sha256Digest)> {
        let link = read_link(&path)?;
        let hash = Sha256::digest(link.as_os_str().as_bytes());

        Ok((path, hash))
    }

    fn hash_items(
        files: Vec<PathBuf>,
        op: fn(PathBuf) -> Result<(PathBuf, Sha256Digest)>,
    ) -> Result<Vec<(PathBuf, GenericArray<u8, sha2::digest::consts::U32>)>> {
        let mut results = Vec::with_capacity(files.len());

        // TODO: Add an impl that uses io_uring to read the contents
        files.into_par_iter().map(op).collect_into_vec(&mut results);

        let mut files = Vec::with_capacity(results.len());
        for result in results {
            files.push(result?)
        }

        Ok(files)
    }

    /// Generates a digest from all the portage files in the repository set.
    pub fn digest(&self) -> Result<RepositoryDigest> {
        // create a Sha256 object
        let mut hasher = Sha256::new();

        let mut files = Vec::<PathBuf>::new();
        let mut symlinks = Vec::<PathBuf>::new();
        for overlay in self.get_repos() {
            for entry in WalkDir::new(overlay.base_dir())
                .into_iter()
                .filter_entry(|e| !Self::ignore_filter(e))
            {
                let entry = entry?;
                let file_type = entry.file_type();
                if file_type.is_dir() {
                    continue;
                } else if file_type.is_symlink() {
                    symlinks.push(entry.into_path());
                } else if file_type.is_file() {
                    files.push(entry.into_path());
                } else {
                    bail!("{} has unknown type", entry.into_path().display());
                }
            }
        }

        let mut files = Self::hash_items(files, Self::hash_file)?;
        let symlinks = Self::hash_items(symlinks, Self::hash_symlink)?;

        files.extend(symlinks);

        // Strip off the root_dir so that we generate the same hash between
        // multiple users. i.e., strip off the users home directory.
        for (name, _hash) in &mut files {
            *name = name.strip_prefix(&self.root_dir)?.to_owned();
        }
        files.sort_by(|a, b| a.0.cmp(&b.0));

        for (name, hash) in &files {
            hasher.update(name.as_os_str().as_bytes());
            hasher.update(hash);
        }

        Ok(RepositoryDigest {
            file_hashes: files,
            repo_hash: hasher.finalize(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct RepositoryLookup {
    root_dir: PathBuf,
    repository_roots: Vec<String>,
}

impl RepositoryLookup {
    /// Uses the specified paths to construct a repository lookup table.
    ///
    /// # Arguments
    ///
    /// * `root_dir` - The root src directory that contains all the
    ///   `repository_roots`.
    /// * `repository_roots` - A list of root paths (relative to the `root_dir`)
    ///    that contain multiple repositories.
    pub fn new(root_dir: &Path, repository_roots: Vec<&str>) -> Result<Self> {
        Ok(RepositoryLookup {
            root_dir: root_dir.to_owned(),
            repository_roots: repository_roots
                .into_iter()
                .map(|s| s.to_owned())
                .collect_vec(),
        })
    }

    /// Find the path for the repository
    pub fn path(&self, repository_name: &str) -> Result<PathBuf> {
        // So we cheat a little bit here. Instead of parsing all of the
        // layout.conf files and generating a hashmap, we rely on the naming
        // convention of the directories. This keeps the initialization cost
        // down since we can avoid scanning a bunch of directories at startup.
        // We validate the layout.conf names when generating the repository set
        // so I think this is a valid optimization.
        for base in &self.repository_roots {
            // This applies to the board overlays.
            let prefixed = format!("overlay-{repository_name}");
            let project = format!("project-{repository_name}");

            // chromiumos is the only repository following this convention.
            let suffixed = format!("{repository_name}-overlay");

            for dir in &[repository_name, &prefixed, &suffixed, &project] {
                let repository_base = self.root_dir.join(base).join(dir);
                let layout = repository_base.join("metadata/layout.conf");

                if layout
                    .try_exists()
                    .with_context(|| format!("checking path {layout:?}"))?
                {
                    return Ok(repository_base);
                }
            }
        }

        bail!(
            "Could not find path for repository named '{}'",
            repository_name
        )
    }

    pub fn create_repository_set(&self, repository_name: &str) -> Result<RepositorySet> {
        let mut layout_map = HashMap::<String, RepositoryLayout>::new();

        let mut remaining = VecDeque::from([repository_name.to_owned()]);
        while let Some(repository_name) = remaining.pop_front() {
            let path = self.path(&repository_name)?;
            let layout = RepositoryLayout::load(&path)?;
            if layout.name != repository_name {
                bail!(
                    "Repository {} has the unexpected name {}, expected {}",
                    path.display(),
                    layout.name,
                    repository_name
                );
            }

            if let Some(old) = layout_map.insert(repository_name.to_owned(), layout) {
                panic!("BUG: Duplicates item {old:?} found in {layout_map:?}");
            }

            // In order to avoid duplicating the parents vector let's just look
            // up the layout object we just put in the map.
            let layout = layout_map.get(&repository_name).expect("layout to exist");

            for parent in &layout.parents {
                if !layout_map.contains_key(parent) && !remaining.contains(parent) {
                    remaining.push_back(parent.to_owned());
                }
            }
        }

        let mut ts = TopologicalSort::<String>::new();
        for (name, layout) in layout_map.iter() {
            ts.insert(name);
            for parent in &layout.parents {
                ts.add_dependency(parent, name);
            }
        }

        let order = ts.into_iter().collect();

        // Finally, build a map from repository names to Repository objects,
        // resolving references.
        let repos: HashMap<String, Repository> = layout_map
            .keys()
            .map(|name| Repository::new(name, &layout_map))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|repo| (repo.name().to_owned(), repo))
            .collect();

        Ok(RepositorySet {
            root_dir: self.root_dir.clone(),
            repos,
            order,
        })
    }
}

#[cfg(test)]
mod tests {
    use sha2::digest::generic_array::arr;

    use super::*;
    use crate::testutils::write_files;

    const GRUNT_LAYOUT_CONF: &str = r#"
masters = portage-stable chromiumos eclass-overlay baseboard-grunt
profile-formats = portage-2 profile-default-eapi
profile_eapi_when_unspecified = 5-progress
repo-name = grunt
thin-manifests = true
use-manifests = strict
"#;

    const BASEBOARD_GRUNT_LAYOUT_CONF: &str = r#"
cache-format = md5-dict
masters = portage-stable chromiumos eclass-overlay
profile-formats = portage-2 profile-default-eapi
profile_eapi_when_unspecified = 5-progress
repo-name = baseboard-grunt
thin-manifests = true
use-manifests = strict
"#;

    const ZORK_LAYOUT_CONF: &str = r#"
masters = portage-stable chromiumos eclass-overlay baseboard-zork
profile-formats = portage-2 profile-default-eapi
profile_eapi_when_unspecified = 5-progress
repo-name = zork
thin-manifests = true
use-manifests = strict
"#;

    const PORTAGE_STABLE_LAYOUT_CONF: &str = r#"
cache-format = md5-dict
masters = eclass-overlay
profile-formats = portage-2
repo-name = portage-stable
thin-manifests = true
use-manifests = strict
"#;

    const CHROMIUMOS_LAYOUT_CONF: &str = r#"
cache-format = md5-dict
masters = portage-stable eclass-overlay
profile-formats = portage-2
repo-name = chromiumos
thin-manifests = true
use-manifests = strict
"#;

    const ECLASS_LAYOUT_CONF: &str = r#"
cache-format = md5-dict
masters =
profile-formats = portage-2
repo-name = eclass-overlay
thin-manifests = true
use-manifests = true
"#;

    const CHEETS_LAYOUT_CONF: &str = r#"
cache-format = md5-dict
masters = portage-stable chromiumos eclass-overlay
profile-formats = portage-2 profile-default-eapi
profile_eapi_when_unspecified = 5-progress
repo-name = cheets-private
thin-manifests = true
use-manifests = strict

"#;

    #[test]
    fn lookp_repository_path() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(
            dir,
            [
                (
                    "overlays/overlay-grunt/metadata/layout.conf",
                    GRUNT_LAYOUT_CONF,
                ),
                (
                    "overlays/baseboard-grunt/metadata/layout.conf",
                    BASEBOARD_GRUNT_LAYOUT_CONF,
                ),
                (
                    "third_party/chromiumos-overlay/metadata/layout.conf",
                    CHROMIUMOS_LAYOUT_CONF,
                ),
                (
                    "private-overlays/project-cheets-private/metadata/layout.conf",
                    CHEETS_LAYOUT_CONF,
                ),
            ],
        )?;

        let lookup =
            RepositoryLookup::new(dir, vec!["private-overlays", "overlays", "third_party"])?;

        assert_eq!(dir.join("overlays/overlay-grunt"), lookup.path("grunt")?);
        assert_eq!(
            dir.join("overlays/overlay-grunt"),
            lookup.path("overlay-grunt")?
        );

        assert!(lookup.path("overlay-grunt-private").is_err());

        assert_eq!(
            dir.join("overlays/baseboard-grunt"),
            lookup.path("baseboard-grunt")?
        );

        assert_eq!(
            dir.join("third_party/chromiumos-overlay"),
            lookup.path("chromiumos")?
        );

        assert_eq!(
            dir.join("private-overlays/project-cheets-private"),
            lookup.path("cheets-private")?
        );

        Ok(())
    }

    #[test]
    fn create_repository_set() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(
            dir,
            [
                (
                    "overlays/overlay-grunt/metadata/layout.conf",
                    GRUNT_LAYOUT_CONF,
                ),
                (
                    "overlays/baseboard-grunt/metadata/layout.conf",
                    BASEBOARD_GRUNT_LAYOUT_CONF,
                ),
                (
                    "overlays/overlay-zork/metadata/layout.conf",
                    ZORK_LAYOUT_CONF,
                ),
                (
                    "third_party/chromiumos-overlay/metadata/layout.conf",
                    CHROMIUMOS_LAYOUT_CONF,
                ),
                (
                    "third_party/portage-stable/metadata/layout.conf",
                    PORTAGE_STABLE_LAYOUT_CONF,
                ),
                (
                    "third_party/eclass-overlay/metadata/layout.conf",
                    ECLASS_LAYOUT_CONF,
                ),
            ],
        )?;

        let lookup =
            RepositoryLookup::new(dir, vec!["private-overlays", "overlays", "third_party"])?;

        let eclass_repo_set = lookup.create_repository_set("eclass-overlay")?;
        assert_eq!("eclass-overlay", eclass_repo_set.primary().name());
        assert_eq!(
            vec!["eclass-overlay"],
            eclass_repo_set
                .get_repos()
                .into_iter()
                .map(|r| r.name())
                .collect::<Vec<&str>>()
        );
        assert_eq!(
            RepositoryDigest {
                file_hashes: vec![(
                    PathBuf::from("third_party/eclass-overlay/metadata/layout.conf"),
                    arr![u8; 68, 216, 205, 202, 131, 32, 140, 82, 54, 145, 136, 189, 135, 114, 241,74, 246, 22, 0, 63, 58, 189, 59, 9, 227, 180, 17, 66, 58, 162, 196, 22]
                ),],
                repo_hash: arr![u8; 151, 67, 227, 38, 227, 189, 212, 99, 5, 79, 205, 188, 87, 211, 146, 223, 10, 197, 156, 142, 104, 95, 135, 191, 156, 122, 126, 119, 51, 79, 253, 10],
            },
            eclass_repo_set.digest()?
        );

        let chromiumos_repo_set = lookup.create_repository_set("chromiumos")?;
        assert_eq!("chromiumos", chromiumos_repo_set.primary().name());
        assert_eq!(
            vec!["eclass-overlay", "portage-stable", "chromiumos"],
            chromiumos_repo_set
                .get_repos()
                .into_iter()
                .map(|r| r.name())
                .collect::<Vec<&str>>()
        );
        assert_eq!(
            RepositoryDigest {
                file_hashes: vec![
                    (
                        PathBuf::from("third_party/chromiumos-overlay/metadata/layout.conf"),
                        arr![u8; 253, 133, 168, 20, 164, 109, 219, 246, 226, 53, 30, 40, 243, 109, 58, 95, 183, 86, 167, 19, 117, 219, 190, 161, 10, 34, 195, 79, 101, 145, 203, 65]
                    ),
                    (
                        PathBuf::from("third_party/eclass-overlay/metadata/layout.conf"),
                        arr![u8; 68, 216, 205, 202, 131, 32, 140, 82, 54, 145, 136, 189, 135, 114, 241, 74, 246, 22, 0, 63, 58, 189, 59, 9, 227, 180, 17, 66, 58, 162, 196, 22]
                    ),
                    (
                        PathBuf::from("third_party/portage-stable/metadata/layout.conf"),
                        arr![u8; 139, 35, 204, 59, 245, 84, 155, 104, 19, 72, 118, 150, 15, 25, 189, 127, 106, 167, 76, 209, 136, 196, 201, 21, 155, 50, 193, 61, 31, 243, 116, 255]
                    ),
                ],
                repo_hash: arr![u8; 188, 5, 43, 86, 236, 121, 238, 103, 249, 78, 203, 154, 216, 12, 194, 32, 214, 238, 151, 188, 109, 199, 88, 13, 115, 98, 77, 30, 99, 220, 107, 0],
            },
            chromiumos_repo_set.digest()?
        );

        let grunt_repo_set = lookup.create_repository_set("grunt")?;
        assert_eq!("grunt", grunt_repo_set.primary().name());
        assert_eq!(
            vec![
                "eclass-overlay",
                "portage-stable",
                "chromiumos",
                "baseboard-grunt",
                "grunt"
            ],
            grunt_repo_set
                .get_repos()
                .into_iter()
                .map(|r| r.name())
                .collect::<Vec<&str>>()
        );

        // Should fail because baseboard-zork isn't defined
        assert!(lookup.create_repository_set("zork").is_err());

        Ok(())
    }
}
