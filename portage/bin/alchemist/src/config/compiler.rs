// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{
    config::bundle::ConfigBundle, config::makeconf::generate::render_make_conf, fileops::FileOps,
    repository::RepositorySet,
};
use anyhow::Result;
use itertools::Itertools;
use std::path::PathBuf;
use std::{collections::HashSet, iter};

#[derive(Debug)]
pub struct ProfileCompiler<'a> {
    config: &'a ConfigBundle,
    profile_path: PathBuf,

    /// The names of all the USE_EXPAND variables. i.e. PYTHON_TARGETS
    use_expand_keys: HashSet<&'a str>,

    /// Variables that can't be set from make.conf.
    profile_only_variables: HashSet<&'a str>,
}

const MAKE_DEFAULT_VARIABLES: &[&str; 2] = &["PROFILE_ONLY_VARIABLES", "USE_EXPAND"];

const IGNORED_VARIABLES: &[&str; 3] = &[
    // We don't need a global USE declaration because we inject a
    // per-package package.use.
    "USE",
    // BINHOSTs are not required because we don't download packages
    // from inside the container. They also change all the time.
    "FULL_BINHOST",
    "PORTAGE_BINHOST",
    // TODO: Strip out all RESUMECOMMAND* and FETCHCOMMAND* variables
    // since we don't need them.
];

impl<'a> ProfileCompiler<'a> {
    pub fn new(profile: &str, repos: &RepositorySet, config: &'a ConfigBundle) -> Self {
        let env = config.env();

        let use_expand = env.get("USE_EXPAND").map(|s| s.as_str()).unwrap_or("");
        let use_expand_unprefixed = env
            .get("USE_EXPAND_UNPREFIXED")
            .map(|s| s.as_str())
            .unwrap_or("");

        let use_expand_keys = use_expand
            .split_whitespace()
            .chain(use_expand_unprefixed.split_whitespace())
            .collect();

        let profile_only_variables = env
            .get("PROFILE_ONLY_VARIABLES")
            .map(|s| s.as_str())
            .unwrap_or("")
            .split_whitespace()
            // TODO: Update the profile to include this one.
            .chain(iter::once("USE_EXPAND_HIDDEN"))
            .collect();

        ProfileCompiler {
            config,
            // TODO: Refactor this so it's part of the ConfigBundle.
            profile_path: repos.primary().profiles_dir().join(profile),
            use_expand_keys,
            profile_only_variables,
        }
    }

    /// The split between make_default and make_conf is kind of arbitrary.
    /// We could in theory add most variables to make.defaults except
    /// `PORTDIR` and `PORTDIR_OVERLAY`, but it felt strange to have variables
    /// with "${SYSROOT}" values in make.defaults. Thia approach puts profile
    /// only variables, and USE_EXPANDED variables in the make.defaults.
    /// This results in a nice split.
    fn is_make_default_key(&self, key: &str) -> bool {
        MAKE_DEFAULT_VARIABLES.iter().any(|x| *x == key)
            || self.profile_only_variables.iter().any(|x| *x == key)
            // `USE_EXPAND`ed variables must be set in the make.defaults
            // because if you set them in the make.conf, they take
            // precedence and override `package.use`.
            //
            // TODO: Evaluate removing `USE_EXPAND`ed variables completely
            // since portage will regenerate them from the USE flags.
            || self.use_expand_keys.iter().any(|x| *x == key)
    }

    /// Returns the env keys and values that make up the compiled profile's
    /// make.defaults.
    fn make_default(&self) -> Vec<(&str, &str)> {
        self.config
            .env()
            .iter()
            .filter(|(key, _val)| self.is_make_default_key(key))
            .sorted_by_key(|(key, _val)| *key)
            .map(|(key, val)| (key.as_ref(), val.as_ref()))
            .collect()
    }

    /// Returns the env keys and values that make up the compiled profile's
    /// make.conf.
    fn make_conf(&self) -> Vec<(&str, &str)> {
        self.config
            .env()
            .iter()
            .filter(|(key, _val)| !IGNORED_VARIABLES.iter().any(|x| x == key))
            .filter(|(key, _val)| !self.is_make_default_key(key))
            // This is only filtered to make the diff between packages compiled
            // with compiled profiles match packages built using portage
            // computed profiles.
            .filter(|(key, val)| key.as_str() != "ENV_UNSET" || !val.is_empty())
            .sorted_by_key(|(key, _val)| *key)
            .map(|(key, val)| (key.as_ref(), val.as_ref()))
            .collect()
    }

    /// Generates a canonicalized portage config.
    pub fn generate_portage_config(&self) -> Result<Vec<FileOps>> {
        let files = vec![
            FileOps::plainfile(
                "/etc/portage/make.conf",
                render_make_conf(self.make_conf())?,
            ),
            FileOps::plainfile(
                "/etc/portage/profile/make.defaults",
                render_make_conf(self.make_default())?,
            ),
            // TODO: We don't actually need to tell portage what the leaf
            // profile is. The only reason we need it right now is to handle
            // the profile.bashrc case. We could either search the parents, find
            // the ones that contain profile.bashrc and generate a synthetic
            // profile listing them as parents. Or we can just concatenate
            // the profile.bashrc scripts and place them in
            // /etc/portage/profile/profile.bashrc.
            FileOps::symlink("/etc/portage/make.profile", &self.profile_path),
        ];

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigNode;
    use crate::config::ConfigNodeValue;
    use crate::config::SimpleConfigSource;
    use crate::repository::Repository;
    use nom::lib::std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn test_use_expand_logic() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_dir = temp_dir.path();

        let repo = Repository::new_for_testing("test", temp_dir);
        let repos = RepositorySet::new_for_testing(&[repo]);

        let config = ConfigBundle::from_sources([SimpleConfigSource::new(vec![ConfigNode {
            sources: vec![PathBuf::from("<fake>")],
            value: ConfigNodeValue::Vars(HashMap::from_iter([
                ("ARCH".into(), "amd64".into()),
                ("ELIBC".into(), "glibc".into()),
                ("PORTDIR".into(), repos.primary().base_dir().to_string_lossy().into()),
                ("ACCEPT_KEYWORDS".into(), "amd64".into()),
                ("PYTHON_TARGETS".into(), "python3_6".into()),
                ("PROFILE_ONLY_VARIABLES".into(), "ARCH ELIBC IUSE_IMPLICIT USE_EXPAND_IMPLICIT USE_EXPAND_UNPREFIXED USE_EXPAND_VALUES_ARCH USE_EXPAND_VALUES_ELIBC".into()),
                ("USE_EXPAND_UNPREFIXED".into(), "ARCH".into()),
                ("USE_EXPAND".into(), "ELIBC PYTHON_TARGETS".into()),
                ("USE_EXPAND_VALUES_ARCH".into(), "alpha amd64 amd64-fbsd amd64-linux arm arm-linux arm64".into()),
                ("USE_EXPAND_VALUES_ELIBC".into(), "FreeBSD glibc musl".into()),
            ])),
        }])]);

        let profile = "base";

        let compiler = ProfileCompiler::new(profile, &repos, &config);

        assert_eq!(
            compiler.make_default(),
            vec![
                ("ARCH", "amd64"),
                ("ELIBC", "glibc"),
                ("IUSE_IMPLICIT", ""),
                ("PROFILE_ONLY_VARIABLES", "ARCH ELIBC IUSE_IMPLICIT USE_EXPAND_IMPLICIT USE_EXPAND_UNPREFIXED USE_EXPAND_VALUES_ARCH USE_EXPAND_VALUES_ELIBC"),
                ("PYTHON_TARGETS", "python3_6"),
                ("USE_EXPAND", "ELIBC PYTHON_TARGETS"),
                ("USE_EXPAND_HIDDEN", ""),
                ("USE_EXPAND_IMPLICIT", "ARCH"),
                ("USE_EXPAND_UNPREFIXED", "ARCH"),
                ("USE_EXPAND_VALUES_ARCH", "alpha amd64 amd64-fbsd amd64-linux arm arm-linux arm64"),
                ("USE_EXPAND_VALUES_ELIBC", "FreeBSD glibc musl")
            ]
        );

        assert_eq!(
            compiler.make_conf(),
            vec![
                ("ACCEPT_KEYWORDS", "amd64"),
                ("CONFIG_PROTECT", ""),
                ("CONFIG_PROTECT_MASK", ""),
                ("FEATURES", ""),
                ("PORTDIR", &repos.primary().base_dir().to_string_lossy()),
            ]
        );

        Ok(())
    }
}
