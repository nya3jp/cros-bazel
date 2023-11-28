// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::config::{bundle::ConfigBundle, site::SiteSettings};
use anyhow::Context;
use anyhow::{anyhow, bail, Error, Result};
use itertools::Itertools;
use once_cell::sync::Lazy;
use rayon::prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use regex::Regex;
use sha2::digest::generic_array::GenericArray;
use sha2::{Digest, Sha256};
use std::cell::{Ref, RefCell};
use std::collections::HashSet;
use std::fs::{read_link, File};
use std::io;
use std::os::unix::prelude::OsStrExt;
use std::{
    borrow::Borrow,
    collections::HashMap,
    ffi::OsStr,
    fs::read_to_string,
    io::ErrorKind,
    iter,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

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
    /// The list of parent repository locations (aka "masters"), in the order
    /// from the least to the most preferred one.
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

    pub fn new_for_testing(name: &str, base_dir: &Path) -> Self {
        Self {
            name: name.to_owned(),
            location: RepositoryLocation::new(base_dir),
            parents: vec![],
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn base_dir(&self) -> &Path {
        &self.location.base_dir
    }

    /// Returns directories to be used for searching eclass files.
    ///
    /// Returned paths are sorted so that a lower-priority eclass directory
    /// comes before a higher-priority one.
    pub fn eclass_dirs(&self) -> impl Iterator<Item = &Path> {
        // Note the "parents" field ("masters" in the overlay layout) is already
        // ordered in the "later entries take precedence" order.
        self.parents
            .iter()
            .map(|location| location.eclass_dir.borrow())
            .chain(iter::once(self.location.eclass_dir.borrow()))
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

pub trait RepositorySetOperations<'a> {
    type Iter: IntoIterator<Item = &'a Repository>;

    fn get_unordered_repos(&'a self) -> Self::Iter;

    /// Looks up a repository by its name.
    fn get_repo_by_name(&'a self, name: &str) -> Result<&'a Repository>;

    /// Looks up a repository that contains the specified file path.
    /// It can be used, for example, to look up a repository that contains an
    /// ebuild file.
    fn get_repo_by_path(&'a self, path: &Path) -> Result<&'a Repository> {
        if !path.is_absolute() {
            bail!(
                "BUG: absolute path required to lookup repositories: {}",
                path.display()
            );
        }
        for repo in self.get_unordered_repos() {
            if path.starts_with(repo.base_dir()) {
                return Ok(repo);
            }
        }
        bail!("repository not found under {}", path.display());
    }
}

/// Holds a set of at least one [`Repository`].
#[derive(Clone, Debug)]
pub struct RepositorySet {
    repos: HashMap<String, Repository>,
    // Keeps the insertion order of `repos`.
    order: Vec<String>,
}

impl<'a> RepositorySetOperations<'a> for RepositorySet {
    type Iter = std::collections::hash_map::Values<'a, String, Repository>;

    fn get_unordered_repos(&'a self) -> Self::Iter {
        self.repos.values()
    }

    fn get_repo_by_name(&self, name: &str) -> Result<&Repository> {
        self.repos
            .get(name)
            .ok_or_else(|| anyhow!("repository not found: {}", name))
    }
}

impl RepositorySet {
    pub fn new_for_testing(repos: &[Repository]) -> Self {
        let mut order: Vec<String> = Vec::new();
        let mut repos_map: HashMap<String, Repository> = HashMap::new();
        for repo in repos {
            order.push(repo.name.clone());
            repos_map.insert(repo.name.clone(), repo.clone());
        }
        Self {
            repos: repos_map,
            order,
        }
    }

    /// Loads repositories configured for a configuration root directory.
    ///
    /// It evaluates `make.conf` in configuration directories tunder `root_dir`
    /// to locate the primary repository (from `$PORTDIR`) and secondary
    /// repositories (from `$PORTDIR_OVERLAY`), and then loads those
    /// repositories.
    pub fn load(root_dir: &Path) -> Result<Self> {
        // Locate repositories by reading PORTDIR and PORTDIR_OVERLAY in make.conf.
        let site_settings = SiteSettings::load(root_dir)?;
        let bootstrap_config = ConfigBundle::from_sources(vec![site_settings]);

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

        Ok(Self { repos, order })
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
            .iter()
            // TODO (b/293383461): The amd64-host profile lists "chromeos" and
            // "chromeos-partner" as the last repositories. e.g.,
            // * portage-stable
            // * x-crossdev
            // * toolchains
            // * chromiumos
            // * eclass-overlay
            // * amd64-host
            // * chromeos-partner
            // * chromeos
            // We really want to return the `amd64-host` repository as the
            // "primary" one since that's the one that contains the profile.
            // In order to correctly fix this, we need to figure out how to
            // correctly identify the "primary" repo using the `board`
            // parameter.
            .filter(|name| !["chromeos", "chromeos-partner"].contains(&name.as_str()))
            .last()
            .expect("repository set should not be empty");
        self.get_repo_by_name(name).unwrap()
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
}

#[derive(Debug, Eq, PartialEq)]
pub struct RepositoryDigest {
    pub file_hashes: Vec<(PathBuf, Sha256Digest)>,
    pub repo_hash: Sha256Digest,
}

impl RepositoryDigest {
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
        let mut hasher = Sha256::new();
        let mut current = path.clone();
        loop {
            current = read_link(&current)?;
            hasher.update(current.as_os_str().as_bytes());

            if !current.try_exists()? {
                break;
            }
            let attr = std::fs::symlink_metadata(&current)?;
            if !attr.is_symlink() {
                if attr.is_file() {
                    let mut file =
                        File::open(&current).with_context(|| "Failed to open {current:?}")?;
                    io::copy(&mut file, &mut hasher)
                        .with_context(|| "Failed to read {current:?}")?;
                }
                break;
            }
        }
        let hash = hasher.finalize();
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
    pub fn new(
        repos: &UnorderedRepositorySet,
        additional_files: Vec<&Path>,
    ) -> Result<RepositoryDigest> {
        // create a Sha256 object
        let mut hasher = Sha256::new();

        let mut files: Vec<_> = additional_files
            .into_iter()
            .map(|p| p.to_path_buf())
            .collect();
        let mut symlinks = Vec::<PathBuf>::new();
        for dir in repos.repos.iter().map(|overlay| overlay.base_dir()) {
            for entry in WalkDir::new(dir)
                .follow_links(true)
                .into_iter()
                .filter_entry(|e| !Self::ignore_filter(e))
            {
                if let Err(e) = &entry {
                    if let Some(io_error) = e.io_error() {
                        if io_error.kind() == std::io::ErrorKind::NotFound {
                            // Handle dangling symlinks.
                            let path = e.path().unwrap();
                            if std::fs::symlink_metadata(path)
                                .with_context(|| format!("{}", path.display()))?
                                .is_symlink()
                            {
                                symlinks.push(path.to_path_buf());
                            }
                            continue;
                        }
                    }
                }
                let entry = entry?;
                let file_type = entry.file_type();
                if entry.path_is_symlink() {
                    symlinks.push(entry.into_path());
                } else if file_type.is_dir() {
                    continue;
                } else if file_type.is_file() {
                    files.push(entry.into_path());
                } else {
                    bail!("{} has unknown type", entry.into_path().display());
                }
            }
        }

        // Ensure we don't hash a file twice.
        files.sort();
        files.dedup();

        let mut files = Self::hash_items(files, Self::hash_file)?;
        let symlinks = Self::hash_items(symlinks, Self::hash_symlink)?;

        files.extend(symlinks);
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

/// Helper struct to make recursion easier
#[derive(Debug)]
struct RepositoryLookupContext {
    seen: HashSet<String>,
    order: Vec<String>,
    try_private: bool,
}

#[derive(Debug)]
pub struct RepositoryLookup {
    root_dir: PathBuf,
    repository_roots: Vec<String>,
    layout_map_cache: RefCell<HashMap<String, RepositoryLayout>>,
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
            layout_map_cache: RefCell::new(HashMap::new()),
        })
    }

    /// Find the path for the repository
    /// Returns None if the repository was not found.
    fn path(&self, repository_name: &str) -> Result<Option<PathBuf>> {
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
                    return Ok(Some(repository_base));
                }
            }
        }

        Ok(None)
    }

    /// Populates an entry in self.layout_map_cache if the repository exists.
    fn load_from_cache(&self, repository_name: &str) -> Result<Option<Ref<RepositoryLayout>>> {
        if let Ok(value) =
            Ref::filter_map(self.layout_map_cache.borrow(), |m| m.get(repository_name))
        {
            return Ok(Some(value));
        };

        let path = match self.path(repository_name)? {
            Some(path) => path,
            None => return Ok(None),
        };

        let layout = RepositoryLayout::load(&path)?;

        if layout.name != repository_name {
            bail!(
                "Repository {} has the unexpected name {}, expected {}",
                path.display(),
                layout.name,
                repository_name
            );
        }

        self.layout_map_cache
            .borrow_mut()
            .insert(repository_name.to_string(), layout);

        // avoids duplicating the borrow code above
        self.load_from_cache(repository_name)
    }

    fn _add_repo(
        &self,
        context: &mut RepositoryLookupContext,
        repo_name: &str,
        required: bool,
    ) -> Result<()> {
        if context.seen.contains(repo_name) {
            return Ok(());
        }
        context.seen.insert(repo_name.to_string());

        let layout = match self.load_from_cache(repo_name)? {
            Some(layout) => layout,
            None => {
                if required {
                    bail!("Failed to find repository {repo_name}");
                } else {
                    return Ok(());
                }
            }
        };

        let mut repos = Vec::new();

        for repo_name in &layout.parents {
            // The extra `context.seen` checks are added as an optimization
            // to reduce the number of allocations required.
            if context.try_private {
                if !context.seen.contains(repo_name) {
                    repos.push((repo_name.to_string(), true));
                }

                if !repo_name.ends_with("-private") {
                    let private_name = format!("{repo_name}-private");

                    // While we have already allocated private_name, we
                    // still check `seen` for consistency and to possibly
                    // avoid allocating space in `repos`.
                    if !context.seen.contains(&private_name) {
                        repos.push((private_name, false));
                    }
                }
            } else {
                if repo_name.ends_with("-private") {
                    bail!("Found private repo in public repos's parent list");
                }
                if !context.seen.contains(repo_name) {
                    repos.push((repo_name.to_string(), true));
                }
            }
        }

        // We need to make sure we drop layout before calling _add_repo since
        // it might modify the cache.
        drop(layout);

        for (repo_name, required) in repos {
            self._add_repo(context, &repo_name, required)?;
        }

        context.order.push(repo_name.to_string());
        Ok(())
    }

    /// Creates a repository set using the provided repository name.
    ///
    /// This is a very ChromeOS specific function. If the repository name
    /// ends in -private, the non suffixed repository will be traversed first.
    /// This ensures that the private repositories have a higher priority
    /// than the public repositories. If the repository name doesn't contain
    /// the -private suffix, it will traverse the `masters` attribute as
    /// left to right.
    ///
    /// This function is not aware of the [PORTDIR](https://wiki.gentoo.org/wiki/PORTDIR)
    /// variable so the order of the main repository (portage-stable) is purely
    /// determined by the order of the `masters` attribute in the layout.conf.
    /// That means the repository set returned here is only suitable for
    /// generating the PORTDIR_OVERLAY variable.
    ///
    /// See https://chromium.googlesource.com/chromiumos/docs/+/HEAD/portage/overlay_faq.md#eclass_overlay
    /// for more information.
    pub fn create_repository_set(&self, repository_name: &str) -> Result<RepositorySet> {
        let mut context = RepositoryLookupContext {
            seen: HashSet::new(),
            order: Vec::new(),
            try_private: repository_name.ends_with("-private"),
        };

        // Try to load the public repo first so it has lower priority
        // than the private repo. It is not guaranteed to exist, so it is marked
        // as optional.
        if let Some(public_name) = repository_name.strip_suffix("-private") {
            self._add_repo(&mut context, public_name, false)?;
        }

        // This is required because we always want to ensure that the
        // `repository_name` that was passed in exists.
        self._add_repo(&mut context, repository_name, true)?;

        // Finally, build a map from repository names to Repository objects,
        // resolving references.
        let repos: HashMap<String, Repository> = context
            .order
            .iter()
            .map(|name| Repository::new(name, &self.layout_map_cache.borrow()))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|repo| (repo.name().to_owned(), repo))
            .collect();

        Ok(RepositorySet {
            repos,
            order: context.order,
        })
    }
}

#[derive(Clone, Debug)]
pub struct UnorderedRepositorySet {
    repos: Vec<Repository>,
}

impl<'a> RepositorySetOperations<'a> for UnorderedRepositorySet {
    type Iter = std::slice::Iter<'a, Repository>;

    fn get_unordered_repos(&'a self) -> Self::Iter {
        self.repos.iter()
    }

    fn get_repo_by_name(&'a self, name: &str) -> Result<&'a Repository> {
        self.repos
            .iter()
            .find(|repo| repo.name() == name)
            .with_context(|| format!("repository not found: {}", name))
    }
}

impl FromIterator<Repository> for UnorderedRepositorySet {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Repository>,
    {
        UnorderedRepositorySet {
            repos: iter
                .into_iter()
                .unique_by(|repo| repo.name().to_string())
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use sha2::digest::generic_array::arr;

    use super::*;
    use crate::testutils::write_files;

    const GRUNT_LAYOUT_CONF: &str = r#"
cache-format = md5-dict
masters = portage-stable chromiumos eclass-overlay baseboard-grunt
profile-formats = portage-2 profile-default-eapi
profile_eapi_when_unspecified = 5-progress
repo-name = grunt
thin-manifests = true
use-manifests = strict
"#;

    const GRUNT_PRIVATE_LAYOUT_CONF: &str = r#"
cache-format = md5-dict
masters = portage-stable chromiumos eclass-overlay grunt baseboard-grunt-private cheets-private
profile-formats = portage-2 profile-default-eapi
profile_eapi_when_unspecified = 5-progress
repo-name = grunt-private
thin-manifests = true
use-manifests = strict
"#;

    const BASEBOARD_GRUNT_LAYOUT_CONF: &str = r#"
cache-format = md5-dict
masters = portage-stable chromiumos eclass-overlay chipset-stnyridge
profile-formats = portage-2 profile-default-eapi
profile_eapi_when_unspecified = 5-progress
repo-name = baseboard-grunt
thin-manifests = true
use-manifests = strict
"#;

    const BASEBOARD_GRUNT_PRIVATE_LAYOUT_CONF: &str = r#"
cache-format = md5-dict
masters = portage-stable chromiumos eclass-overlay baseboard-grunt chipset-stnyridge-private
profile-formats = portage-2 profile-default-eapi
profile_eapi_when_unspecified = 5-progress
repo-name = baseboard-grunt-private
thin-manifests = true
use-manifests = strict
"#;

    const CHIPSET_STNYRIDGE_LAYOUT_CONF: &str = r#"
cache-format = md5-dict
masters = portage-stable chromiumos eclass-overlay
profile-formats = portage-2 profile-default-eapi
profile_eapi_when_unspecified = 5-progress
repo-name = chipset-stnyridge
thin-manifests = true
use-manifests = strict
"#;

    const CHIPSET_STNYRIDGE_PRIVATE_LAYOUT_CONF: &str = r#"
cache-format = md5-dict
masters = portage-stable chromiumos eclass-overlay chipset-stnyridge
profile-formats = portage-2 profile-default-eapi
profile_eapi_when_unspecified = 5-progress
repo-name = chipset-stnyridge-private
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

    const CHEETS_PRIVATE_LAYOUT_CONF: &str = r#"
cache-format = md5-dict
masters = portage-stable chromiumos eclass-overlay
profile-formats = portage-2 profile-default-eapi
profile_eapi_when_unspecified = 5-progress
repo-name = cheets-private
thin-manifests = true
use-manifests = strict
"#;

    #[test]
    fn lookup_repository_path() -> Result<()> {
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
                    CHEETS_PRIVATE_LAYOUT_CONF,
                ),
            ],
        )?;

        let lookup =
            RepositoryLookup::new(dir, vec!["private-overlays", "overlays", "third_party"])?;

        assert_eq!(
            Some(dir.join("overlays/overlay-grunt")),
            lookup.path("grunt")?
        );
        assert_eq!(
            Some(dir.join("overlays/overlay-grunt")),
            lookup.path("overlay-grunt")?
        );

        assert_eq!(None, lookup.path("overlay-grunt-private")?);

        assert_eq!(
            Some(dir.join("overlays/baseboard-grunt")),
            lookup.path("baseboard-grunt")?
        );

        assert_eq!(
            Some(dir.join("third_party/chromiumos-overlay")),
            lookup.path("chromiumos")?
        );

        assert_eq!(
            Some(dir.join("private-overlays/project-cheets-private")),
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
                    "private-overlays/overlay-grunt-private/metadata/layout.conf",
                    GRUNT_PRIVATE_LAYOUT_CONF,
                ),
                (
                    "overlays/baseboard-grunt/metadata/layout.conf",
                    BASEBOARD_GRUNT_LAYOUT_CONF,
                ),
                (
                    "private-overlays/baseboard-grunt-private/metadata/layout.conf",
                    BASEBOARD_GRUNT_PRIVATE_LAYOUT_CONF,
                ),
                (
                    "overlays/chipset-stnyridge/metadata/layout.conf",
                    CHIPSET_STNYRIDGE_LAYOUT_CONF,
                ),
                (
                    "private-overlays/chipset-stnyridge-private/metadata/layout.conf",
                    CHIPSET_STNYRIDGE_PRIVATE_LAYOUT_CONF,
                ),
                (
                    "private-overlays/project-cheets-private/metadata/layout.conf",
                    CHEETS_PRIVATE_LAYOUT_CONF,
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
            vec![(
                dir.join("third_party/eclass-overlay/metadata/layout.conf"),
                arr![u8; 68, 216, 205, 202, 131, 32, 140, 82, 54, 145, 136, 189, 135, 114, 241, 74,
                246, 22, 0, 63, 58, 189, 59, 9, 227, 180, 17, 66, 58, 162, 196, 22]
            ),],
            RepositoryDigest::new(
                &(eclass_repo_set.get_repos().into_iter().cloned().collect()),
                vec![]
            )?
            .file_hashes
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
            vec![
                (
                    dir.join("third_party/chromiumos-overlay/metadata/layout.conf"),
                    arr![u8; 253, 133, 168, 20, 164, 109, 219, 246, 226, 53, 30, 40, 243, 109, 58,
                    95, 183, 86, 167, 19, 117, 219, 190, 161, 10, 34, 195, 79, 101, 145, 203, 65]
                ),
                (
                    dir.join("third_party/eclass-overlay/metadata/layout.conf"),
                    arr![u8; 68, 216, 205, 202, 131, 32, 140, 82, 54, 145, 136, 189, 135, 114, 241,
                    74, 246, 22, 0, 63, 58, 189, 59, 9, 227, 180, 17, 66, 58, 162, 196, 22]
                ),
                (
                    dir.join("third_party/portage-stable/metadata/layout.conf"),
                    arr![u8; 139, 35, 204, 59, 245, 84, 155, 104, 19, 72, 118, 150, 15, 25, 189,
                    127, 106, 167, 76, 209, 136, 196, 201, 21, 155, 50, 193, 61, 31, 243, 116, 255]
                ),
            ],
            RepositoryDigest::new(
                &(chromiumos_repo_set
                    .get_repos()
                    .into_iter()
                    .cloned()
                    .collect()),
                vec![]
            )?
            .file_hashes
        );

        let grunt_repo_set = lookup.create_repository_set("grunt")?;
        assert_eq!("grunt", grunt_repo_set.primary().name());
        assert_eq!(
            vec![
                "eclass-overlay",
                "portage-stable",
                "chromiumos",
                "chipset-stnyridge",
                "baseboard-grunt",
                "grunt"
            ],
            grunt_repo_set
                .get_repos()
                .into_iter()
                .map(|r| r.name())
                .collect::<Vec<&str>>()
        );

        let grunt_private_repo_set = lookup.create_repository_set("grunt-private")?;
        assert_eq!("grunt-private", grunt_private_repo_set.primary().name());
        // This list differs from `emerge-grunt --info --verbose` because the
        // board's make.conf explicitly overrides the PORTDIR_OVERLAY order:
        // PORTDIR_OVERLAY="
        //   /mnt/host/source/src/third_party/chromiumos-overlay
        //   /mnt/host/source/src/third_party/eclass-overlay
        //   ${BOARD_OVERLAY}
        // "
        assert_eq!(
            vec![
                "eclass-overlay",
                "portage-stable",
                "chromiumos",
                "chipset-stnyridge",
                "chipset-stnyridge-private",
                "baseboard-grunt",
                "baseboard-grunt-private",
                "grunt",
                "cheets-private",
                "grunt-private",
            ],
            grunt_private_repo_set
                .get_repos()
                .into_iter()
                .map(|r| r.name())
                .collect::<Vec<&str>>()
        );

        // Should fail because baseboard-zork isn't defined
        assert!(lookup.create_repository_set("zork").is_err());

        let repos: UnorderedRepositorySet = [&grunt_repo_set, &grunt_private_repo_set]
            .into_iter()
            .flat_map(|set| set.get_repos())
            .cloned()
            .collect();

        assert_eq!(
            HashSet::from([
                "eclass-overlay",
                "portage-stable",
                "chromiumos",
                "chipset-stnyridge",
                "chipset-stnyridge-private",
                "baseboard-grunt",
                "baseboard-grunt-private",
                "grunt",
                "cheets-private",
                "grunt-private",
            ]),
            repos
                .get_unordered_repos()
                .map(|repo| repo.name())
                .collect(),
        );

        assert_eq!(
            "portage-stable",
            repos.get_repo_by_name("portage-stable")?.name()
        );

        assert_eq!(
            "baseboard-grunt-private",
            repos
                .get_repo_by_path(&dir.join(
                    "private-overlays/baseboard-grunt-private/sys-libs/glibc/glibc-1.0.ebuild"
                ))?
                .name()
        );

        Ok(())
    }
}
