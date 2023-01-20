// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Context;
use anyhow::{anyhow, bail, Error, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    borrow::Borrow,
    collections::{HashMap, VecDeque},
    ffi::OsStr,
    fs::read_to_string,
    io::ErrorKind,
    iter,
    path::{Path, PathBuf},
};

use topological_sort::TopologicalSort;

use crate::{
    config::{bundle::ConfigBundle, site::SiteSettings, ConfigSource},
    data::Vars,
};

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
        let content = read_to_string(&path).with_context(|| format!("reading {path:?}"))?;

        let mut name: Option<String> = None;
        let mut parents = Vec::<String>::new();

        for (lineno, line) in content.split('\n').enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let caps = LAYOUT_CONF_LINE_RE.captures(line).ok_or_else(|| {
                anyhow!("{}:{}: syntax error", path.to_string_lossy(), lineno + 1)
            })?;
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

        let name =
            name.ok_or_else(|| anyhow!("{}: repo-name not defined", path.to_string_lossy()))?;
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
            location: location.clone(),
            parents,
        })
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

        Ok(ebuild_paths)
    }
}

/// Holds a set of at least one [`Repository`].
#[derive(Clone, Debug)]
pub struct RepositorySet {
    repos: HashMap<String, Repository>,
    // Keeps the insertion order of `repos`.
    order: Vec<String>,
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
            .map(|v| v.clone())
            .ok_or_else(|| anyhow!("PORTDIR is not defined in system configs"))?;
        let secondary_repo_dirs = bootstrap_config
            .env()
            .get("PORTDIR_OVERLAY")
            .map(|v| v.clone())
            .unwrap_or_default();

        // Read layout.conf in repositories to build a map from repository names
        // to repository layout info.
        let mut layout_map = HashMap::<String, RepositoryLayout>::new();
        let mut order: Vec<String> = Vec::new();
        for repo_dir in iter::once(primary_repo_dir.borrow())
            .chain(secondary_repo_dirs.split_ascii_whitespace())
        {
            let repo_dir = PathBuf::from(repo_dir);
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
    pub fn find_ebuilds(&self, package_name: &str) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::<PathBuf>::new();
        for repo in self.repos.values() {
            paths.extend(repo.find_ebuilds(package_name)?);
        }
        Ok(paths)
    }

    /// Scans the repositories and returns all ebuild file paths.
    pub fn find_all_ebuilds(&self) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::<PathBuf>::new();
        for repo in self.repos.values() {
            paths.extend(repo.find_all_ebuilds()?);
        }
        Ok(paths)
    }
}

#[derive(Clone, Debug)]
pub struct RepositoryLookup {
    repository_roots: Vec<PathBuf>,
}

impl RepositoryLookup {
    /// Uses the specified paths to construct a repository lookup table.
    ///
    /// # Arguments
    ///
    /// * `repository_roots` - A list of root paths that contain multiple repositories.
    pub fn new(repository_roots: Vec<PathBuf>) -> Result<Self> {
        Ok(RepositoryLookup { repository_roots })
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

            // chromiumos is the only repository following this convention.
            let suffixed = format!("{repository_name}-overlay");

            for dir in &[repository_name, &prefixed, &suffixed] {
                let repository_base = base.join(dir);
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

        Ok(RepositorySet { repos, order })
    }
}

#[cfg(test)]
mod tests {
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
            ],
        )?;

        let lookup = RepositoryLookup::new(vec![
            dir.join("private-overlays"),
            dir.join("overlays"),
            dir.join("third_party"),
        ])?;

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

        let lookup = RepositoryLookup::new(vec![
            dir.join("private-overlays"),
            dir.join("overlays"),
            dir.join("third_party"),
        ])?;

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
