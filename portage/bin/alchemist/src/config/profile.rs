// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use itertools::Itertools;
use std::{fs::read_to_string, io::ErrorKind, path::Path};

use crate::{
    config::{
        makeconf::MakeConf,
        miscconf::{
            accept_keywords::load_accept_keywords_configs, mask::load_package_configs,
            provided::load_provided_packages_config, useflags::load_use_configs,
        },
        ConfigNode, ConfigSource,
    },
    data::Vars,
    path::clean_path,
    repository::RepositorySet,
};

/// Parsed Portage profile.
#[derive(Debug, Eq, PartialEq)]
pub struct Profile {
    parents: Vec<Profile>,
    makeconf: MakeConf,
    precomputed_nodes: Vec<ConfigNode>,
}

impl Profile {
    /// Loads a Portage profile located at `dir`.
    ///
    /// `repos` is a set of known repositories. It is used to resolve profile
    /// parents.
    pub fn load(dir: &Path, repos: &RepositorySet) -> Result<Self> {
        let context = || format!("Failed to load profile {}", dir.display());
        let parent_keys = load_parents(&dir.join("parent"))?;
        let parents = parent_keys
            .into_iter()
            .map(|parent_key| {
                let parent_dir = if let Some((repo_name, rel_path)) = parent_key.split_once(':') {
                    let parent_repo = repos.get_repo_by_name(repo_name)?;
                    parent_repo.profiles_dir().join(rel_path)
                } else {
                    clean_path(&dir.join(&parent_key))?
                };
                let profile = Self::load(&parent_dir, repos)?;
                Ok(profile)
            })
            .collect::<Result<Vec<_>>>()
            .with_context(context)?;
        let makeconf =
            MakeConf::load(&dir.join("make.defaults"), dir, false, true).with_context(context)?;

        let precomputed_nodes = [
            load_package_configs(dir).with_context(context)?,
            load_accept_keywords_configs(dir).with_context(context)?,
            load_use_configs(dir).with_context(context)?,
            load_provided_packages_config(dir).with_context(context)?,
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
        let symlink_path = root_dir.join("etc/portage/make.profile");
        let dir = symlink_path
            .read_link()
            .with_context(|| format!("Reading symlink at {}", symlink_path.display()))?;
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
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        parents.push(line.to_owned());
    }
    Ok(parents)
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs::create_dir_all, os::unix::fs::symlink};
    use tempfile::tempdir;

    use super::*;

    use crate::{repository::Repository, testutils::write_files};

    #[test]
    fn test_load() -> Result<()> {
        let dir = tempdir()?;
        let dir = dir.as_ref();

        const AMD64_GEN_MAKE_CONF: &str = concat!(
            "mnt/host/source/",
            "src/overlays/overlay-amd64-generic/profiles/",
            "base/make.defaults"
        );

        const CHROMIUM_MAKE_CONF: &str = concat!(
            "mnt/host/source/",
            "src/third_party/chromiumos-overlay/profiles/",
            "default/linux/amd64/10.0/chromeos/make.defaults"
        );

        write_files(
            dir,
            [
                (AMD64_GEN_MAKE_CONF, "A=aa\n"),
                (
                    "mnt/host/source/src/overlays/overlay-amd64-generic/profiles/base/parent",
                    "chromiumos:default/linux/amd64/10.0/chromeos\nchromiumos:features/selinux\n",
                ),
                (CHROMIUM_MAKE_CONF, "B=bb\n"),
            ],
        )?;

        create_dir_all(dir.join("build/amd64-generic/etc/portage"))?;
        symlink(
            dir.join("mnt/host/source/src/overlays/overlay-amd64-generic/profiles/base"),
            dir.join("build/amd64-generic/etc/portage/make.profile"),
        )?;

        let repos = RepositorySet::new_for_testing(&[Repository::new_for_testing(
            "chromiumos",
            dir.join("mnt/host/source/src/third_party/chromiumos-overlay")
                .as_path(),
        )]);

        let repo_root = dir.join("build/amd64-generic");
        let actual = Profile::load_default(repo_root.as_path(), &repos)?;

        let expected = Profile {
            parents: vec![
                Profile {
                    parents: vec![],
                    makeconf: MakeConf::new_for_testing(
                        vec![dir.join(CHROMIUM_MAKE_CONF)],
                        HashMap::from([("B", "bb")]),
                    ),
                    precomputed_nodes: vec![],
                },
                Profile {
                    parents: vec![],
                    makeconf: MakeConf::new_for_testing(vec![], HashMap::new()),
                    precomputed_nodes: vec![],
                },
            ],
            makeconf: MakeConf::new_for_testing(
                vec![dir.join(AMD64_GEN_MAKE_CONF)],
                HashMap::from([("A", "aa")]),
            ),
            precomputed_nodes: vec![],
        };

        assert_eq!(expected, actual);

        Ok(())
    }
}
