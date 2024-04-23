// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{
    config::bundle::ConfigBundle, config::makeconf::generate::render_make_conf, fileops::FileOps,
};
use anyhow::{bail, Result};
use itertools::Itertools;
use std::path::Path;
use std::{collections::HashSet, iter};

#[derive(Debug)]
pub struct ProfileCompiler<'a> {
    config: &'a ConfigBundle,

    /// The names of all the USE_EXPAND variables. i.e. PYTHON_TARGETS
    use_expand_keys: HashSet<&'a str>,

    /// Variables that can't be set from make.conf.
    profile_only_variables: HashSet<&'a str>,

    sysroot: &'a Path,

    /// If set, any variables starting with with sysroot will be omitted / rewritten.
    strip_sysroot: bool,
}

const MAKE_DEFAULT_VARIABLES: &[&str; 2] = &["PROFILE_ONLY_VARIABLES", "USE_EXPAND"];

const IGNORED_VARIABLES: &[&str; 3] = &[
    // We don't need a global USE declaration because we inject a
    // per-package package.use.
    "USE",
    // We don't inject ACCEPT_LICENSE because it's not evaluated when
    // invoking ebuild.
    "ACCEPT_LICENSE",
    // If MAKEOPTS isn't set, Portage will default it to `-j<cores>`.
    "MAKEOPTS",
    // TODO: Strip out all RESUMECOMMAND* and FETCHCOMMAND* variables
    // since we don't need them.
];

// When generating a host (ROOT=/) profile from a target/host
// (ROOT=/build/amd64-host) profile, we need to modify some of the variables.
// A None means to omit the value, a Some means to use this value instead.
const TARGET_HOST_TO_HOST_VARIABLES: &[(&str, Option<&str>); 5] = &[
    ("PKG_CONFIG", None),
    ("PKGDIR", Some("/var/lib/portage/pkgs")),
    ("PORT_LOGDIR", Some("/var/log/portage")),
    ("PORTAGE_TMPDIR", Some("/var/tmp")),
    ("ROOT", None),
];

impl<'a> ProfileCompiler<'a> {
    pub fn new(config: &'a ConfigBundle, sysroot: &'a Path) -> Self {
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
            use_expand_keys,
            profile_only_variables,
            sysroot,
            strip_sysroot: false,
        }
    }

    /// Strips the specified `sysroot` from the generated config.
    ///
    /// This function is used when generating a "host" config bundle from a
    /// "target/host" config bundle. The "target/host" config bundle
    /// contains variables like `ROOT="/build/amd64-host"`,
    /// `PKGDIR="/build/amd64-host//packages/"`, etc. These paths are invalid
    /// when building for the "host", so we need to strip and/or modify them.
    pub fn strip_sysroot(mut self, value: bool) -> Self {
        self.strip_sysroot = value;

        self
    }

    /// Applies the `TARGET_HOST_TO_HOST_VARIABLES` map to the specified`key`
    /// and `value`.
    fn map_sysroot_variable(&'a self, key: &'a str, value: &'a str) -> Option<(&'a str, &'a str)> {
        for (k, v) in TARGET_HOST_TO_HOST_VARIABLES {
            if key == *k {
                return v.map(|v| (key, v));
            }
        }
        Some((key, value))
    }

    /// Checks if the value starts with the sysroot.
    fn is_sysroot_variable(&self, value: &str) -> bool {
        Path::new(value).starts_with(self.sysroot)
    }

    /// Returns an error if the value matches the sysroot.
    ///
    /// This is used to guard against any sysroot specific variables from
    /// leaking into the host config.
    fn error_if_sysroot_variable(
        &'a self,
        key: &'a str,
        value: &'a str,
    ) -> Result<(&'a str, &'a str)> {
        if self.is_sysroot_variable(value) {
            bail!("Profile variables should be sysroot agnostic. Got {key}={value}")
        } else {
            Ok((key, value))
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

    fn is_ignored_key(&self, key: &str) -> bool {
        IGNORED_VARIABLES.iter().any(|x| *x == key) ||
        // BINHOSTs are not required because we don't download packages
        // from inside the container. They also change all the time.
        key.ends_with("_BINHOST")
    }

    /// Returns the env keys and values that make up the compiled profile's
    /// make.defaults.
    fn make_default(&self) -> Result<Vec<(&str, &str)>> {
        self.config
            .env()
            .iter()
            .filter(|(key, _val)| self.is_make_default_key(key))
            .sorted_by_key(|(key, _val)| *key)
            .map(|(key, val)| self.error_if_sysroot_variable(key, val))
            .collect()
    }

    /// Returns the env keys and values that make up the compiled profile's
    /// make.conf.
    fn make_conf(&self) -> Result<Vec<(&str, &str)>> {
        self.config
            .env()
            .iter()
            .filter(|(key, _val)| !self.is_ignored_key(key))
            .filter(|(key, _val)| !self.is_make_default_key(key))
            // This is only filtered to make the diff between packages compiled
            // with compiled profiles match packages built using portage
            // computed profiles.
            .filter(|(key, val)| key.as_str() != "ENV_UNSET" || !val.is_empty())
            .filter_map(|(key, val)| {
                if self.strip_sysroot {
                    self.map_sysroot_variable(key, val)
                } else {
                    Some((key.as_str(), val.as_str()))
                }
            })
            .sorted_by_key(|(key, _val)| *key)
            .map(|(key, val)| {
                if self.strip_sysroot {
                    self.error_if_sysroot_variable(key, val)
                } else {
                    Ok((key, val))
                }
            })
            .collect()
    }

    /// Generates a canonicalized portage config.
    pub fn generate_portage_config(&self) -> Result<Vec<FileOps>> {
        let files = vec![
            FileOps::plainfile(
                // Ideally this would be placed in /etc/portage/make.conf,
                // but it turns out that chromite's sysroot_lib has
                // /etc/make.conf hard coded. When generating licenses chromite
                // will "source" the /etc/make.conf file to calculate the
                // PORTDIR_OVERLAY. It then search for the license in each of
                // those repositories. We should fix chromite to support
                // /etc/portage/make.conf.
                "/etc/make.conf",
                render_make_conf(self.make_conf()?)?,
            ),
            FileOps::plainfile(
                "/etc/portage/make.profile/make.defaults",
                render_make_conf(self.make_default()?)?,
            ),
        ];

        Ok(files)
    }

    /// Returns the env keys and values that make up the compiled profile's
    /// make.conf.
    fn make_conf_lite(&self) -> Vec<(&str, &str)> {
        let allowed_vars = HashSet::from(["ARCH", "CHOST"]);

        self.config
            .env()
            .iter()
            .filter(|(key, _val)| allowed_vars.contains(key.as_str()))
            .sorted_by_key(|(key, _val)| *key)
            .map(|(key, val)| (key.as_str(), val.as_str()))
            .collect()
    }

    /// Generates a lite portage config.
    ///
    /// When doing a cross-root build, the host config doesn't actually come
    /// into play, so we generate a very simple config to appease portage.
    pub fn generate_lite_portage_config(&self) -> Result<Vec<FileOps>> {
        let files = vec![
            FileOps::plainfile(
                "/etc/portage/make.conf",
                render_make_conf(self.make_conf_lite())?,
            ),
            FileOps::plainfile("/etc/portage/make.profile/make.defaults", ""),
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
    use crate::repository::RepositoryLayout;
    use crate::repository::RepositorySet;
    use crate::repository::RepositorySetOperations;
    use nom::lib::std::collections::HashMap;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_use_expand_logic() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_dir = temp_dir.path();

        let repos = RepositorySet::load_from_layouts(
            "test",
            &[RepositoryLayout::new("test", temp_dir, &[])],
        )?;
        let repo = repos.get_repo_by_name("test")?;

        let sysroot = "/build/amd64-host";

        let config = ConfigBundle::from_sources([SimpleConfigSource::new(vec![ConfigNode {
            sources: vec![PathBuf::from("<fake>")],
            value: ConfigNodeValue::Vars(HashMap::from_iter([
                ("ARCH".into(), "amd64".into()),
                ("CHOST".into(), "x86_64-pc-linux-gnu".into()),
                ("ELIBC".into(), "glibc".into()),
                ("PORTDIR".into(), repo.base_dir().to_string_lossy().into()),
                ("ACCEPT_KEYWORDS".into(), "amd64".into()),
                ("ACCEPT_LICENSE".into(), "* -@EULA".into()),
                ("PYTHON_TARGETS".into(), "python3_6".into()),
                ("PROFILE_ONLY_VARIABLES".into(), "ARCH ELIBC IUSE_IMPLICIT USE_EXPAND_IMPLICIT USE_EXPAND_UNPREFIXED USE_EXPAND_VALUES_ARCH USE_EXPAND_VALUES_ELIBC".into()),
                ("USE_EXPAND_UNPREFIXED".into(), "ARCH".into()),
                ("USE_EXPAND".into(), "ELIBC PYTHON_TARGETS".into()),
                ("USE_EXPAND_VALUES_ARCH".into(), "alpha amd64 amd64-fbsd amd64-linux arm arm-linux arm64".into()),
                ("USE_EXPAND_VALUES_ELIBC".into(), "FreeBSD glibc musl".into()),
                ("ROOT".into(), sysroot.into()),
                ("MAKEOPTS".into(), "-j 32".into()),
                ("PKG_CONFIG".into(), format!("{sysroot}/build/bin/pkg-config")),
                ("CQ_BINHOST".into(), "http://foo".to_string()),
            ])),
        }])]);

        let mut compiler = ProfileCompiler::new(&config, Path::new(sysroot));

        assert_eq!(
            compiler.make_default()?,
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
            compiler.make_conf()?,
            vec![
                ("ACCEPT_KEYWORDS", "amd64"),
                ("CHOST", "x86_64-pc-linux-gnu"),
                ("CONFIG_PROTECT", ""),
                ("CONFIG_PROTECT_MASK", ""),
                ("FEATURES", ""),
                ("PKG_CONFIG", &format!("{sysroot}/build/bin/pkg-config")),
                ("PORTDIR", &repo.base_dir().to_string_lossy()),
                ("ROOT", sysroot),
            ]
        );

        assert_eq!(
            compiler.make_conf_lite(),
            vec![("ARCH", "amd64"), ("CHOST", "x86_64-pc-linux-gnu"),]
        );

        compiler = compiler.strip_sysroot(true);

        assert_eq!(
            compiler.make_default()?,
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
            compiler.make_conf()?,
            vec![
                ("ACCEPT_KEYWORDS", "amd64"),
                ("CHOST", "x86_64-pc-linux-gnu"),
                ("CONFIG_PROTECT", ""),
                ("CONFIG_PROTECT_MASK", ""),
                ("FEATURES", ""),
                ("PORTDIR", &repo.base_dir().to_string_lossy()),
            ]
        );

        Ok(())
    }
}
