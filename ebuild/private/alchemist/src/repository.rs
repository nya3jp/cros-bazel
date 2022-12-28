// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{anyhow, bail, Error, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    borrow::Borrow,
    collections::HashMap,
    ffi::OsStr,
    fs::read_to_string,
    io::ErrorKind,
    iter,
    path::{Path, PathBuf},
};

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
        let content = read_to_string(&path)?;

        let mut name: Option<String> = None;
        let mut parents = Vec::<String>::new();

        for (lineno, line) in content.split('\n').enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("#") {
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

/// Holds a set of [`Repository`].
#[derive(Clone, Debug)]
pub struct RepositorySet {
    repos: HashMap<String, Repository>,
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
            .ebuild_env()
            .get("PORTDIR")
            .map(|v| v.clone())
            .ok_or_else(|| anyhow!("PORTDIR is not defined in system configs"))?;
        let secondary_repo_dirs = bootstrap_config
            .ebuild_env()
            .get("PORTDIR_OVERLAY")
            .map(|v| v.clone())
            .unwrap_or_default();

        // Read layout.conf in repositories to build a map from repository names
        // to repository layout info.
        let mut layout_map = HashMap::<String, RepositoryLayout>::new();
        for repo_dir in iter::once(primary_repo_dir.borrow())
            .chain(secondary_repo_dirs.split_ascii_whitespace())
        {
            let repo_dir = PathBuf::from(repo_dir);
            let layout = RepositoryLayout::load(&repo_dir)?;
            if let Some(old_layout) = layout_map.insert(layout.name.to_owned(), layout) {
                bail!(
                    "multiple repositories have the same name: {}",
                    old_layout.name
                );
            }
        }

        // Finally, build a map from repository names to Repository objects,
        // resolving references.
        let repos = layout_map
            .keys()
            .map(|name| Repository::new(name, &layout_map))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|repo| (repo.name().to_owned(), repo))
            .collect();

        Ok(Self { repos })
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
