// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use itertools::Itertools;
use std::{fs::read_to_string, io::ErrorKind, path::Path};

use crate::{
    config::{
        makeconf::MakeConf,
        miscconf::{
            mask::load_package_configs, provided::load_provided_packages_config,
            useflags::load_use_configs,
        },
        ConfigNode, ConfigSource,
    },
    data::Vars,
    repository::RepositorySet,
};

/// Parsed Portage profile.
#[derive(Debug)]
pub struct Profile {
    parents: Vec<Box<Profile>>,
    makeconf: MakeConf,
    precomputed_nodes: Vec<ConfigNode>,
}

impl Profile {
    /// Loads a Portage profile located at `dir`.
    ///
    /// `repos` is a set of known repositories. It is used to resolve profile
    /// parents.
    pub fn load(dir: &Path, repos: &RepositorySet) -> Result<Self> {
        let parent_keys = load_parents(&dir.join("parent"))?;
        let parents = parent_keys
            .into_iter()
            .map(|parent_key| {
                let parent_dir = if let Some((repo_name, rel_path)) = parent_key.split_once(':') {
                    let parent_repo = repos.get_repo_by_name(repo_name)?;
                    parent_repo.profiles_dir().join(rel_path)
                } else {
                    dir.join(&parent_key)
                };
                let profile = Self::load(&parent_dir, repos)?;
                Ok(Box::new(profile))
            })
            .collect::<Result<Vec<_>>>()?;
        let makeconf = MakeConf::load(&dir.join("make.defaults"), dir, false, true)?;

        let precomputed_nodes = [
            load_package_configs(dir)?,
            load_use_configs(dir)?,
            load_provided_packages_config(dir)?,
        ]
        .concat();

        Ok(Self {
            parents,
            makeconf,
            precomputed_nodes,
        })
    }

    /// Loads the default Portage profile for the configuration root `root_dir`.
    ///
    /// It is a short-hand for loading `${root_dir}/etc/portage/make.profile`,
    /// after resolving the symlink.
    pub fn load_default(root_dir: &Path, repos: &RepositorySet) -> Result<Self> {
        let dir = root_dir.join("etc/portage/make.profile").read_link()?;
        Profile::load(&dir, repos)
    }
}

impl ConfigSource for Profile {
    fn evaluate_configs(&self, env: &mut Vars) -> Vec<ConfigNode> {
        let mut nodes = self
            .parents
            .iter()
            .flat_map(|parent| parent.evaluate_configs(env))
            .collect_vec();
        nodes.extend(self.makeconf.evaluate_configs(env));
        nodes.extend(self.precomputed_nodes.clone());
        nodes
    }
}

fn load_parents(path: &Path) -> Result<Vec<String>> {
    let contents = read_to_string(path).or_else(|err| {
        if err.kind() == ErrorKind::NotFound {
            Ok(String::new())
        } else {
            Err(err)
        }
    })?;
    let mut parents = Vec::<String>::new();
    for line in contents.split('\n') {
        let line = line.trim();
        if line.is_empty() || line.starts_with("#") {
            continue;
        }
        parents.push(line.to_owned());
    }
    Ok(parents)
}
