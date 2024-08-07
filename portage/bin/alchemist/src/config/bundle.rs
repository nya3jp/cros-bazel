// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::HashMap,
    collections::HashSet,
    iter,
    path::{Path, PathBuf},
};

use anyhow::Result;
use itertools::Itertools;
use version::Version;

use crate::{
    bash::vars::BashVars,
    data::{IUseMap, Slot, UseMap, Vars},
    dependency::package::PackageRef,
};

use super::{
    ConfigNode, ConfigNodeValue, ConfigSource, PackageMaskKind, ProvidedPackage,
    SimpleConfigSource, UseUpdateKind,
};

struct BuiltinIncrementalVariable {
    name: &'static str,
    defaults: &'static str,
}

// A list of profile variables treated as incremental by default.
// https://projects.gentoo.org/pms/8/pms.html#x1-560005.3.1
// USE and ACCEPT_KEYWORDS are not listed here because they can be affected by configs and handled
// separately.
const BUILTIN_INCREMENTAL_VARIABLES: &[BuiltinIncrementalVariable] = &[
    BuiltinIncrementalVariable {
        name: "USE_EXPAND",
        defaults: "",
    },
    BuiltinIncrementalVariable {
        name: "USE_EXPAND_HIDDEN",
        defaults: "",
    },
    BuiltinIncrementalVariable {
        name: "CONFIG_PROTECT",
        defaults: "",
    },
    BuiltinIncrementalVariable {
        name: "CONFIG_PROTECT_MASK",
        defaults: "",
    },
    // Portage treats this as an incremental variable. We can't do that because
    // we don't want to copy the default that portage uses. Instead we treat
    // this one special in `compute_general_incremental_variable` and just do
    // a simple concatenation so that portage can compute the real value using
    // its defaults.
    BuiltinIncrementalVariable {
        name: "FEATURES",
        defaults: "",
    },
    BuiltinIncrementalVariable {
        name: "IUSE_IMPLICIT",
        defaults: "",
    },
    // PMS does not require handling ARCH specially for EAPI 5+, but do it
    // anyway for compatibility.
    BuiltinIncrementalVariable {
        name: "USE_EXPAND_IMPLICIT",
        defaults: "ARCH",
    },
    BuiltinIncrementalVariable {
        name: "USE_EXPAND_UNPREFIXED",
        defaults: "ARCH",
    },
    BuiltinIncrementalVariable {
        name: "ENV_UNSET",
        defaults: "",
    },
];

/// Merges incremental variable tokens as defined in PMS.
/// https://projects.gentoo.org/pms/8/pms.html#x1-560005.3.1
///
/// Returned tokens are sorted.
fn merge_incremental_tokens<'s, I: IntoIterator<Item = &'s str>>(
    iter: I,
) -> impl Iterator<Item = &'s str> {
    let mut values = HashSet::<&str>::new();
    for token in iter {
        if let Some(token) = token.strip_prefix('-') {
            if token == "*" {
                values.clear();
            } else {
                values.remove(token);
            }
        } else {
            values.insert(token);
        }
    }
    values.into_iter().sorted()
}

/// Represents a result of ConfigBundle::is_package_accepted().
pub enum IsPackageAcceptedResult {
    /// The package is not accepted.
    Unaccepted { reason: String },
    /// The package is accepted. The boolean value is true if the package is considered stable.
    Accepted { stable: bool },
}

/// A collection of [`ConfigNode`]s, providing access to the configurations
/// computed from them.
#[derive(Clone, Debug)]
pub struct ConfigBundle {
    nodes: Vec<ConfigNode>,
    env: Vars,
    incremental_variables: HashMap<String, Vec<String>>,
    use_expand_values: Vec<String>,
    provided_packages: Vec<ProvidedPackage>,
}

impl ConfigBundle {
    /// Creates [`ConfigBundle`] from underlying [`ConfigSource`]s.
    pub fn from_sources<S: ConfigSource, I: IntoIterator<Item = S>>(sources: I) -> Self {
        let mut env = Vars::new();
        let nodes = sources
            .into_iter()
            .flat_map(|source| source.evaluate_configs(&mut env))
            .collect::<Vec<_>>();

        // Compute incremental variables that are not specific to packages.
        let incremental_variables = Self::compute_general_incremental_variables(&nodes);

        env.extend(
            incremental_variables
                .iter()
                .map(|(key, tokens)| (key.clone(), tokens.join(" "))),
        );

        // Compute USE flags originated from USE_EXPAND/USE_EXPAND_UNPREFIXED.
        let use_expand_prefixed = incremental_variables
            .get("USE_EXPAND")
            .unwrap()
            .iter()
            .flat_map(|name| {
                let lower_name = name.to_ascii_lowercase();
                Self::compute_general_incremental_variable(&nodes, name, "")
                    .map(move |s| format!("{}_{}", &lower_name, s))
            });
        let use_expand_unprefixed = incremental_variables
            .get("USE_EXPAND_UNPREFIXED")
            .unwrap()
            .iter()
            .flat_map(|name| {
                Self::compute_general_incremental_variable(&nodes, name, "").map(|s| s.to_owned())
            });
        let use_expand_values = use_expand_prefixed
            .chain(use_expand_unprefixed)
            .collect_vec();

        // Compute provided packages.
        let provided_packages = nodes
            .iter()
            .flat_map(|node| match &node.value {
                ConfigNodeValue::ProvidedPackages(packages) => packages.clone(),
                _ => Vec::new(),
            })
            .collect_vec();

        Self {
            nodes,
            env,
            incremental_variables,
            use_expand_values,
            provided_packages,
        }
    }

    /// Creates a minimal [`ConfigBundle`] suitable for unit testing.
    pub fn new_for_testing(arch: &str) -> Self {
        Self::from_sources([SimpleConfigSource::new(vec![ConfigNode {
            sources: vec![PathBuf::from("<fake>")],
            value: ConfigNodeValue::Vars(HashMap::from_iter([
                ("ARCH".into(), arch.into()),
                ("ACCEPT_KEYWORDS".into(), arch.into()),
                // GENTOO_MIRRORS is required for remote source analysis.
                ("GENTOO_MIRRORS".into(), "http://localhost/gentoo".into()),
            ])),
        }])])
    }

    /// Returns variables defined by underlying sources.
    ///
    /// Incremental variables are already resolved. Use [`compute_use_map`] to
    /// compute USE flags for a package, instead of reading `USE` variable with
    /// this method, since `USE` flags can vary by packages and thus it makes
    /// little sense to compute "global USE flags". The same goes for
    /// ACCEPT_KEYWORDS. Use [`compute_accept_keywords`] instead of this method.
    ///
    /// This is often called as "profile variables", even though they can be
    /// defined in non-profile sources such as make.conf.
    pub fn env(&self) -> &Vars {
        &self.env
    }

    /// Computes ACCEPT_KEYWORDS of a package.
    fn compute_accept_keywords(
        nodes: &[ConfigNode],
        default_for_empty_config_line: &str,
        package: &PackageRef,
    ) -> Vec<String> {
        let config_values = nodes
            .iter()
            .flat_map(|node| match &node.value {
                ConfigNodeValue::Vars(vars) => vars
                    .get("ACCEPT_KEYWORDS")
                    .map_or(Vec::new(), |value| vec![&**value]),
                ConfigNodeValue::AcceptKeywords(updates) => updates
                    .iter()
                    .filter(|update| update.atom.matches(package))
                    .map(|o| {
                        if o.accept_keywords.is_empty() {
                            default_for_empty_config_line
                        } else {
                            o.accept_keywords.as_str()
                        }
                    })
                    .collect(),
                _ => Vec::new(),
            })
            .flat_map(|s| s.split_ascii_whitespace());

        merge_incremental_tokens(config_values)
            .map(|s| s.to_owned())
            .collect()
    }

    fn is_keyword_accepted<T1: AsRef<str>, T2: AsRef<str>>(
        keywords: &[T1],
        accept_keywords: &[T2],
    ) -> bool {
        // Visit each accepted keyword.
        for accept in accept_keywords.iter().map(|x| x.as_ref()) {
            // "**" as an accepted keyword matches with anything including empty keywords.
            if accept == "**" {
                return true;
            }
            // Visit each keyword.
            for keyword in keywords.iter().map(|x| x.as_ref()) {
                if keyword.starts_with('-') {
                    // Ignore broken keywords.
                    continue;
                }
                if keyword == "*" {
                    // "*" as a keyword matches with any accepted keyword.
                    return true;
                } else if keyword == "~*" {
                    // "~*" as a keyword matches with any accepted keyword starting with "~".
                    if accept.starts_with('~') {
                        return true;
                    }
                } else if keyword == accept {
                    // Exact match.
                    return true;
                } else if keyword.starts_with('~') {
                    // A keyword starting with "~" matches with "~*" as an accepted keyword.
                    if accept == "~*" {
                        return true;
                    }
                } else {
                    // A keyword not starting with "~" matches with "*" as an accepted keyword.
                    if accept == "*" {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Returns if a package is accepted by checking KEYWORDS and ACCEPT_KEYWORDS.
    pub fn is_package_accepted(
        &self,
        vars: &BashVars,
        package: &PackageRef,
    ) -> Result<IsPackageAcceptedResult> {
        // ~$ARCH is used as the default value for an empty config line.
        let arch = self.env().get("ARCH").map(|s| &**s).unwrap_or_default();
        let default_for_empty_config_line = format!("~{arch}");

        let accept_keywords =
            Self::compute_accept_keywords(&self.nodes, &default_for_empty_config_line, package);
        let keywords = vars
            .get_scalar_or_default("KEYWORDS")?
            .split_ascii_whitespace()
            .collect_vec();

        if !Self::is_keyword_accepted(&keywords, &accept_keywords) {
            return Ok(IsPackageAcceptedResult::Unaccepted {
                reason: format!(
                    "KEYWORDS ({}) is not accepted by ACCEPT_KEYWORDS ({})",
                    keywords.join(" "),
                    accept_keywords.join(" ")
                ),
            });
        }

        // A package is considered stable if adding "~" to all stable keywords results in not
        // accepting the package. See the explanation about "stable restrictions" in Package
        // Manager Specification 5.2.11.
        let modified_keywords = keywords
            .into_iter()
            .map(|keyword| {
                if keyword.starts_with('~') {
                    keyword.to_owned()
                } else {
                    format!("~{keyword}")
                }
            })
            .collect_vec();
        let stable = !Self::is_keyword_accepted(&modified_keywords, &accept_keywords);
        Ok(IsPackageAcceptedResult::Accepted { stable })
    }

    /// Computes USE flags of a package.
    pub fn compute_use_map(
        &self,
        package_name: &str,
        version: &Version,
        stable: bool,
        slot: &Slot<String>,
        ebuild_iuse_map: &IUseMap,
    ) -> UseMap {
        let package = &PackageRef {
            package_name,
            version,
            slot: Some(Slot {
                main: slot.main.as_ref(),
                sub: slot.sub.as_ref(),
            }),
            use_map: None,
            readiness: None,
        };

        let effective_iuse_map = self.compute_effective_iuse_map(ebuild_iuse_map);

        let all_use_set: HashSet<&str> = self
            .compute_use_variable_for_package(package, stable, &effective_iuse_map)
            .collect();
        let all_use_mask: HashSet<&str> = self
            .compute_use_masks(package, stable, UseUpdateKind::Mask)
            .collect();
        let all_use_force: HashSet<&str> = self
            .compute_use_masks(package, stable, UseUpdateKind::Force)
            .collect();

        UseMap::from_iter(effective_iuse_map.keys().map(|name| {
            let mut value = all_use_set.contains(name.as_str());

            // Apply mask/force. If both are applied, the mask takes precedence.
            // https://projects.gentoo.org/pms/8/pms.html#x1-540005.2.11
            if all_use_mask.contains(name.as_str()) {
                value = false;
            } else if all_use_force.contains(name.as_str()) {
                value = true;
            }

            (name.to_owned(), value)
        }))
    }

    /// Returns if a package is masked by package.mask and friends.
    pub fn is_package_masked(&self, package: &PackageRef) -> bool {
        let status = self
            .nodes
            .iter()
            .flat_map(|node| match &node.value {
                ConfigNodeValue::PackageMasks(updates) => updates.as_slice(),
                _ => &[],
            })
            .filter_map(|update| {
                if update.atom.matches(package) {
                    Some(update.kind)
                } else {
                    None
                }
            })
            .last()
            .unwrap_or(PackageMaskKind::Unmask);
        status == PackageMaskKind::Mask
    }

    /// Returns a list of package declared as "provided" by package.provided.
    pub fn provided_packages(&self) -> &Vec<ProvidedPackage> {
        &self.provided_packages
    }

    /// Returns a list of all the configuration sources.
    pub fn sources(&self) -> Vec<&Path> {
        self.nodes
            .iter()
            .flat_map(|node| &node.sources)
            .map(|path| path.as_path())
            .collect()
    }

    /// Returns the bashrc files that need to be executed with the package.
    pub fn package_bashrcs(&self, package: &PackageRef) -> Vec<PathBuf> {
        let mut paths = vec![];

        for node in &self.nodes {
            match &node.value {
                ConfigNodeValue::ProfileBashrc(bashrcs) => paths.extend(bashrcs.iter().cloned()),
                ConfigNodeValue::PackageBashrcs(bashrcs) => paths.extend(
                    bashrcs
                        .iter()
                        .filter(|bashrc| bashrc.atom.matches(package))
                        .flat_map(|bashrc| &bashrc.paths)
                        .cloned(),
                ),
                _ => continue,
            }
        }

        paths
    }

    /// Returns all profile.bashrc files defined by the profile.
    pub fn all_profile_bashrcs(&self) -> Vec<&Path> {
        let mut paths = vec![];

        for node in &self.nodes {
            match &node.value {
                ConfigNodeValue::ProfileBashrc(bashrcs) => {
                    paths.extend(bashrcs.iter().map(|path| path.as_path()))
                }
                _ => continue,
            }
        }

        paths
    }

    /// Returns all package.bashrc files defined by the profile.
    pub fn all_package_bashrcs(&self) -> Vec<&Path> {
        let mut paths = vec![];

        for node in &self.nodes {
            match &node.value {
                ConfigNodeValue::PackageBashrcs(bashrcs) => paths.extend(
                    bashrcs
                        .iter()
                        .flat_map(|bashrc| &bashrc.paths)
                        .map(|path| path.as_path()),
                ),
                _ => continue,
            }
        }

        paths
    }

    /// Computes the effective IUSE of a package, which includes IUSE explicitly
    /// defined in ebuild/eclass and profile-injected IUSE.
    ///
    /// The effective IUSE is defined as IUSE_EFFECTIVE/IUSE_REFERENCEABLE in
    /// the PMS.
    /// https://projects.gentoo.org/pms/8/pms.html#x1-11000011.1.1
    fn compute_effective_iuse_map(&self, ebuild_iuse_map: &IUseMap) -> IUseMap {
        let mut effective_iuse_map = IUseMap::new();

        let iuse_implicit: Vec<&str> = self
            .incremental_variables
            .get("IUSE_IMPLICIT")
            .unwrap()
            .iter()
            .map(|s| s.as_str())
            .collect();
        let use_expand_prefixed: HashSet<&str> = self
            .incremental_variables
            .get("USE_EXPAND")
            .unwrap()
            .iter()
            .map(|s| s.as_str())
            .collect();
        let use_expand_unprefixed: HashSet<&str> = self
            .incremental_variables
            .get("USE_EXPAND_UNPREFIXED")
            .unwrap()
            .iter()
            .map(|s| s.as_str())
            .collect();
        let use_expand_implicit: HashSet<&str> = self
            .incremental_variables
            .get("USE_EXPAND_IMPLICIT")
            .unwrap()
            .iter()
            .map(|s| s.as_str())
            .collect();

        for token in iuse_implicit.iter() {
            effective_iuse_map.insert((*token).to_owned(), false);
        }

        for expand_token in use_expand_prefixed.intersection(&use_expand_implicit) {
            for token in self
                .env
                .get(&format!("USE_EXPAND_VALUES_{}", *expand_token))
                .map(|s| &**s)
                .unwrap_or_default()
                .split_ascii_whitespace()
            {
                effective_iuse_map.insert(
                    format!("{}_{}", expand_token.to_ascii_lowercase(), token),
                    false,
                );
            }
        }

        for expand_token in use_expand_unprefixed.intersection(&use_expand_implicit) {
            for token in self
                .env
                .get(&format!("USE_EXPAND_VALUES_{}", *expand_token))
                .map(|s| &**s)
                .unwrap_or_default()
                .split_ascii_whitespace()
            {
                effective_iuse_map.insert(token.to_owned(), false);
            }
        }

        effective_iuse_map.extend(ebuild_iuse_map.clone());

        effective_iuse_map
    }

    /// Compute the USE flags of a package.
    /// Note that the function does not take USE masks into account. One must
    /// call compute_use_masks as well to compute the actual USE flags exposed
    /// to a package.
    fn compute_use_variable_for_package<'a>(
        &'a self,
        package: &'a PackageRef,
        stable: bool,
        effective_iuse_map: &'a IUseMap,
    ) -> impl Iterator<Item = &'a str> {
        // USE flags originated from IUSE in the ebuild.
        let ebuild_uses = effective_iuse_map.iter().filter_map(|(name, value)| {
            if value == &true {
                Some(name.as_str())
            } else {
                None
            }
        });

        // USE flags originated from configs, e.g. profiles and make.conf.
        let config_uses = self
            .nodes
            .iter()
            .flat_map(move |node| match &node.value {
                ConfigNodeValue::Vars(vars) => {
                    vars.get("USE").map_or(Vec::new(), |value| vec![&**value])
                }
                ConfigNodeValue::Uses(updates) => updates
                    .iter()
                    .filter(|update| {
                        if update.kind != UseUpdateKind::Set {
                            return false;
                        }
                        if let Some(atom) = &update.filter.atom {
                            if !atom.matches(package) {
                                return false;
                            }
                        }
                        if update.filter.stable_only && !stable {
                            return false;
                        }
                        true
                    })
                    .map(|o| o.use_tokens.as_str())
                    .collect(),
                _ => Vec::new(),
            })
            .flat_map(|s| s.split_ascii_whitespace());

        // Compute the actual value by concatenating values from sources.
        merge_incremental_tokens(
            ebuild_uses
                .chain(self.use_expand_values.iter().map(|s| &**s))
                .chain(config_uses),
        )
    }

    /// Compute the masked USE flags of a package.
    fn compute_use_masks<'a>(
        &'a self,
        package: &'a PackageRef,
        stable: bool,
        kind: UseUpdateKind,
    ) -> impl Iterator<Item = &'a str> {
        merge_incremental_tokens(
            self.nodes
                .iter()
                .flat_map(move |node| match &node.value {
                    ConfigNodeValue::Uses(updates) => updates
                        .iter()
                        .filter_map(|update| {
                            if update.kind != kind {
                                return None;
                            }
                            if update.filter.stable_only && !stable {
                                return None;
                            }
                            if let Some(atom) = &update.filter.atom {
                                if !atom.matches(package) {
                                    return None;
                                }
                            }
                            Some(update.use_tokens.as_str())
                        })
                        .collect_vec(),
                    _ => Vec::new(),
                })
                .flat_map(|s| s.split_ascii_whitespace()),
        )
    }

    /// Compute the values of all incremental variables, except USE and ACCEPT_KEYWORDS whose value
    /// varies by package.
    ///
    /// This function is supposed to be called from the constructor and its
    /// result should be cached, thus this function does not take self.
    fn compute_general_incremental_variables(nodes: &[ConfigNode]) -> HashMap<String, Vec<String>> {
        // Compute built-in incremental variables.
        let builtins: HashMap<String, Vec<String>> = BUILTIN_INCREMENTAL_VARIABLES
            .iter()
            .map(|v| {
                let values = Self::compute_general_incremental_variable(nodes, v.name, v.defaults)
                    .map(|value| value.to_owned())
                    .collect_vec();
                (v.name.to_owned(), values)
            })
            .collect();

        // Compute non-built-in incremental variables, namely USE_EXPAND and
        // USE_EXPAND_UNPREFIXED.
        // https://projects.gentoo.org/pms/8/pms.html#x1-570005.3.2
        let expands: HashMap<String, Vec<String>> = ["USE_EXPAND", "USE_EXPAND_UNPREFIXED"]
            .into_iter()
            .flat_map(|name| {
                builtins
                    .get(name)
                    .map(|values| values.as_slice())
                    .unwrap_or_default()
            })
            .map(|name| {
                let values = Self::compute_general_incremental_variable(nodes, name, "")
                    .map(|value| value.to_owned())
                    .collect_vec();
                (name.clone(), values)
            })
            .collect();

        builtins.into_iter().chain(expands).collect()
    }

    /// Compute the value of an incremental variable.
    ///
    /// This function must not be used to compute the value of USE as its value
    /// varies by package. It will panic if [name] is "USE", which should happen
    /// only when USE is contained in USE_EXPAND or USE_EXPAND_UNPREFIXED. The
    /// same goes for ACCEPT_KEYWORDS.
    ///
    /// This function is supposed to be called from the constructor and its
    /// result should be cached, thus this function does not take self.
    fn compute_general_incremental_variable<'a>(
        nodes: &'a [ConfigNode],
        name: &'a str,
        defaults: &'a str,
    ) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        if name == "USE" || name == "ACCEPT_KEYWORDS" {
            panic!("USE_EXPAND/USE_EXPAND_UNPREFIXED must not contain {}", name);
        }
        let tokens = iter::once(defaults)
            .chain(nodes.iter().filter_map(move |node| match &node.value {
                ConfigNodeValue::Vars(vars) => vars.get(name).map(|value| &**value),
                _ => None,
            }))
            .flat_map(|s| s.split_ascii_whitespace());

        if name == "FEATURES" {
            Box::new(tokens)
        } else {
            Box::new(merge_incremental_tokens(tokens))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;
    use std::str::FromStr;

    use lazy_static::lazy_static;

    use crate::{
        config::{
            AcceptKeywordsUpdate, PackageBashrc, SimpleConfigSource, UseUpdate, UseUpdateFilter,
        },
        dependency::package::PackageAtom,
    };

    #[test]
    fn test_compute_accept_keywords() -> Result<()> {
        let package = PackageRef {
            package_name: "aaa/bbb",
            version: &Version::try_new("9999")?,
            slot: Some(Slot {
                main: "0",
                sub: "0",
            }),
            use_map: None,
            readiness: None,
        };
        let default_for_empty_config_line = "~amd64";

        // The default case. Just returns the current arch.
        assert_eq!(
            ConfigBundle::compute_accept_keywords(
                &[ConfigNode {
                    sources: vec![PathBuf::from("a")],
                    value: ConfigNodeValue::Vars(HashMap::from([(
                        "ACCEPT_KEYWORDS".to_owned(),
                        "amd64".to_owned()
                    )])),
                }],
                default_for_empty_config_line,
                &package
            ),
            vec!["amd64"]
        );

        // After cros_workon start.
        assert_eq!(
            ConfigBundle::compute_accept_keywords(
                &[
                    ConfigNode {
                        sources: vec![PathBuf::from("a")],
                        value: ConfigNodeValue::Vars(HashMap::from([(
                            "ACCEPT_KEYWORDS".to_owned(),
                            "amd64".to_owned()
                        )])),
                    },
                    ConfigNode {
                        sources: vec![PathBuf::from("b")],
                        value: ConfigNodeValue::AcceptKeywords(vec![AcceptKeywordsUpdate {
                            atom: PackageAtom::from_str("=aaa/bbb-9999")?,
                            accept_keywords: "".to_owned(),
                        }]),
                    }
                ],
                default_for_empty_config_line,
                &package
            ),
            vec!["amd64", "~amd64"]
        );

        // After cros_workon start, but for a different package.
        assert_eq!(
            ConfigBundle::compute_accept_keywords(
                &[
                    ConfigNode {
                        sources: vec![PathBuf::from("a")],
                        value: ConfigNodeValue::Vars(HashMap::from([(
                            "ACCEPT_KEYWORDS".to_owned(),
                            "amd64".to_owned()
                        )])),
                    },
                    ConfigNode {
                        sources: vec![PathBuf::from("b")],
                        value: ConfigNodeValue::AcceptKeywords(vec![AcceptKeywordsUpdate {
                            atom: PackageAtom::from_str("=ccc/ddd-9999")?,
                            accept_keywords: "".to_owned(),
                        }]),
                    }
                ],
                default_for_empty_config_line,
                &package
            ),
            vec!["amd64"]
        );

        // Non-empty accept_keywords value.
        assert_eq!(
            ConfigBundle::compute_accept_keywords(
                &[
                    ConfigNode {
                        sources: vec![PathBuf::from("a")],
                        value: ConfigNodeValue::Vars(HashMap::from([(
                            "ACCEPT_KEYWORDS".to_owned(),
                            "amd64".to_owned()
                        )])),
                    },
                    ConfigNode {
                        sources: vec![PathBuf::from("b")],
                        value: ConfigNodeValue::AcceptKeywords(vec![AcceptKeywordsUpdate {
                            atom: PackageAtom::from_str("=aaa/bbb-9999")?,
                            accept_keywords: "-* arm64 ~arm64".to_owned(),
                        }]),
                    }
                ],
                default_for_empty_config_line,
                &package
            ),
            vec!["arm64", "~arm64"]
        );

        Ok(())
    }

    #[test]
    fn test_sources() -> Result<()> {
        let bundle = ConfigBundle::from_sources(vec![SimpleConfigSource::new(vec![
            ConfigNode {
                sources: vec![PathBuf::from("a")],
                value: ConfigNodeValue::Vars(HashMap::from([(
                    "ACCEPT_KEYWORDS".to_owned(),
                    "amd64".to_owned(),
                )])),
            },
            ConfigNode {
                sources: vec![PathBuf::from("b")],
                value: ConfigNodeValue::AcceptKeywords(vec![AcceptKeywordsUpdate {
                    atom: PackageAtom::from_str("=aaa/bbb-9999")?,
                    accept_keywords: "".to_owned(),
                }]),
            },
        ])]);

        assert_eq!(bundle.sources(), vec![Path::new("a"), Path::new("b")]);

        Ok(())
    }

    #[test]
    fn test_is_keyword_accepted() -> Result<()> {
        // "**" matches with anything including empty keywords.
        assert!(ConfigBundle::is_keyword_accepted::<&str, &str>(
            &[],
            &["**"]
        ));
        assert!(ConfigBundle::is_keyword_accepted(&["amd64"], &["**"]));
        assert!(ConfigBundle::is_keyword_accepted(&["~amd64"], &["**"]));
        assert!(ConfigBundle::is_keyword_accepted(&["-amd64"], &["**"]));
        assert!(ConfigBundle::is_keyword_accepted(&["*"], &["**"]));
        assert!(ConfigBundle::is_keyword_accepted(&["~*"], &["**"]));
        assert!(ConfigBundle::is_keyword_accepted(&["-*"], &["**"]));

        // "*" as a keyword matches with any accepted keyword.
        assert!(ConfigBundle::is_keyword_accepted(&["*"], &["amd64"]));
        assert!(ConfigBundle::is_keyword_accepted(&["*"], &["~amd64"]));
        assert!(ConfigBundle::is_keyword_accepted(&["*"], &["*"]));
        assert!(ConfigBundle::is_keyword_accepted(&["*"], &["~*"]));

        // "~*" as a keyword matches with any accepted keyword starting with "~".
        assert!(!ConfigBundle::is_keyword_accepted(&["~*"], &["amd64"]));
        assert!(ConfigBundle::is_keyword_accepted(&["~*"], &["~amd64"]));
        assert!(!ConfigBundle::is_keyword_accepted(&["~*"], &["*"]));
        assert!(ConfigBundle::is_keyword_accepted(&["~*"], &["~*"]));

        // A keyword starting with "~".
        assert!(!ConfigBundle::is_keyword_accepted(&["~amd64"], &["amd64"]));
        assert!(ConfigBundle::is_keyword_accepted(&["~amd64"], &["~amd64"]));
        assert!(!ConfigBundle::is_keyword_accepted(&["~amd64"], &["*"]));
        assert!(ConfigBundle::is_keyword_accepted(&["~amd64"], &["~*"]));

        // A keyword starting with "-" doesn't match with anything.
        assert!(!ConfigBundle::is_keyword_accepted(&["-amd64"], &["amd64"]));
        assert!(!ConfigBundle::is_keyword_accepted(&["-amd64"], &["~amd64"]));
        assert!(!ConfigBundle::is_keyword_accepted(&["-amd64"], &["*"]));
        assert!(!ConfigBundle::is_keyword_accepted(&["-amd64"], &["~*"]));

        // Multiple keywords.
        assert!(ConfigBundle::is_keyword_accepted(
            &["amd64", "~arm64"],
            &["amd64"]
        ));
        assert!(!ConfigBundle::is_keyword_accepted(
            &["amd64", "~arm64"],
            &["~amd64"]
        ));
        assert!(!ConfigBundle::is_keyword_accepted(
            &["amd64", "~arm64"],
            &["arm64"]
        ));
        assert!(ConfigBundle::is_keyword_accepted(
            &["amd64", "~arm64"],
            &["~arm64"]
        ));

        // Multiple accepted keywords.
        assert!(ConfigBundle::is_keyword_accepted(
            &["amd64"],
            &["amd64", "~amd64"]
        ));
        assert!(ConfigBundle::is_keyword_accepted(
            &["~amd64"],
            &["amd64", "~amd64"]
        ));
        assert!(!ConfigBundle::is_keyword_accepted(
            &["arm64"],
            &["amd64", "~amd64"]
        ));
        assert!(!ConfigBundle::is_keyword_accepted(
            &["~arm64"],
            &["amd64", "~amd64"]
        ));

        // Empty keywords.
        assert!(!ConfigBundle::is_keyword_accepted::<&str, &str>(
            &[],
            &["amd64"]
        ));
        assert!(!ConfigBundle::is_keyword_accepted::<&str, &str>(
            &[],
            &["~amd64"]
        ));
        assert!(!ConfigBundle::is_keyword_accepted::<&str, &str>(
            &[],
            &["*"]
        ));
        assert!(!ConfigBundle::is_keyword_accepted::<&str, &str>(
            &[],
            &["~*"]
        ));

        // No accepted keywords.
        assert!(!ConfigBundle::is_keyword_accepted::<&str, &str>(
            &["amd64"],
            &[]
        ));
        assert!(!ConfigBundle::is_keyword_accepted::<&str, &str>(
            &["~amd64"],
            &[]
        ));
        assert!(!ConfigBundle::is_keyword_accepted::<&str, &str>(
            &["*"],
            &[]
        ));
        assert!(!ConfigBundle::is_keyword_accepted::<&str, &str>(
            &["~*"],
            &[]
        ));

        Ok(())
    }

    lazy_static! {
        static ref VERSION_9999: Version = Version::try_new("9999").unwrap();
        static ref PACKAGE_REF_A: PackageRef<'static> = PackageRef {
            package_name: "aaa/bbb",
            version: &VERSION_9999,
            slot: Some(Slot {
                main: "0",
                sub: "0",
            }),
            use_map: None,
            readiness: None,
        };
    }

    #[test]
    fn test_use_expand_simple() -> Result<()> {
        let bundle = ConfigBundle::from_sources(vec![SimpleConfigSource::new(vec![ConfigNode {
            sources: vec![PathBuf::from("a")],
            value: ConfigNodeValue::Vars(HashMap::from([
                ("USE_EXPAND".to_owned(), "PYTHON_TARGETS".to_owned()),
                (
                    "PYTHON_TARGETS".to_owned(),
                    "-python2_7 python3_6".to_owned(),
                ),
            ])),
        }])]);

        let iuse = HashMap::from([("python_targets_python3_6".to_string(), false)]);
        let use_flags = bundle
            .compute_use_variable_for_package(&PACKAGE_REF_A, true, &iuse)
            .collect_vec();

        assert_eq!(use_flags, vec!["python_targets_python3_6"]);

        Ok(())
    }

    #[test]
    fn test_use_expand_override() -> Result<()> {
        let bundle = ConfigBundle::from_sources(vec![SimpleConfigSource::new(vec![
            ConfigNode {
                sources: vec![PathBuf::from("make.defaults")],
                value: ConfigNodeValue::Vars(HashMap::from([
                    ("USE_EXPAND".to_owned(), "PYTHON_TARGETS".to_owned()),
                    (
                        "PYTHON_TARGETS".to_owned(),
                        "-python2_7 python3_6".to_owned(),
                    ),
                ])),
            },
            ConfigNode {
                sources: vec![PathBuf::from("package.use")],
                value: ConfigNodeValue::Uses(vec![UseUpdate {
                    kind: UseUpdateKind::Set,
                    filter: UseUpdateFilter {
                        atom: None,
                        stable_only: false,
                    },
                    use_tokens: "-python_targets_python3_6".to_string(),
                }]),
            },
        ])]);

        let iuse = HashMap::from([("python_targets_python3_6".to_string(), false)]);
        let use_flags = bundle
            .compute_use_variable_for_package(&PACKAGE_REF_A, true, &iuse)
            .collect_vec();

        assert_eq!(use_flags, Vec::<String>::new());

        Ok(())
    }

    #[test]
    fn test_mask_use_expand() -> Result<()> {
        let bundle = ConfigBundle::from_sources(vec![SimpleConfigSource::new(vec![
            ConfigNode {
                sources: vec![PathBuf::from("make.defaults")],
                value: ConfigNodeValue::Vars(HashMap::from([
                    ("USE_EXPAND".to_owned(), "VIDEO_CARDS".to_owned()),
                    ("VIDEO_CARDS".to_owned(), "vesa vmware".to_owned()),
                ])),
            },
            ConfigNode {
                sources: vec![PathBuf::from("use.mask.1")],
                value: ConfigNodeValue::Uses(vec![UseUpdate {
                    kind: UseUpdateKind::Mask,
                    filter: UseUpdateFilter {
                        atom: None,
                        stable_only: false,
                    },
                    use_tokens: "video_cards_vmware".to_string(),
                }]),
            },
            ConfigNode {
                sources: vec![PathBuf::from("use.mask.2")],
                value: ConfigNodeValue::Uses(vec![UseUpdate {
                    kind: UseUpdateKind::Mask,
                    filter: UseUpdateFilter {
                        atom: None,
                        stable_only: false,
                    },
                    use_tokens: "-video_cards_vmware".to_string(),
                }]),
            },
        ])]);

        let iuse = HashMap::from([("video_cards_vmware".to_string(), false)]);
        let use_flags = bundle
            .compute_use_variable_for_package(&PACKAGE_REF_A, true, &iuse)
            .collect_vec();

        assert_eq!(
            use_flags,
            [
                "video_cards_vesa".to_string(),
                "video_cards_vmware".to_string()
            ]
        );

        Ok(())
    }

    #[test]
    fn test_features() -> Result<()> {
        let bundle = ConfigBundle::from_sources(vec![SimpleConfigSource::new(vec![
            ConfigNode {
                sources: vec![PathBuf::from("make.defaults")],
                value: ConfigNodeValue::Vars(HashMap::from([(
                    "FEATURES".to_owned(),
                    "buildpkg clean-logs -collision-protect".to_owned(),
                )])),
            },
            ConfigNode {
                sources: vec![PathBuf::from("make.defaults")],
                value: ConfigNodeValue::Vars(HashMap::from([(
                    "FEATURES".to_owned(),
                    "collision-protect -news".to_owned(),
                )])),
            },
        ])]);

        let features = bundle.env().get("FEATURES").unwrap();

        assert_eq!(
            features,
            "buildpkg clean-logs -collision-protect collision-protect -news",
        );

        Ok(())
    }

    #[test]
    fn test_bashrc() -> Result<()> {
        let bundle = ConfigBundle::from_sources(vec![SimpleConfigSource::new(vec![
            ConfigNode {
                sources: vec![
                    PathBuf::from("foo/profile.bashrc/a.sh"),
                    PathBuf::from("foo/profile.bashrc/b.sh"),
                ],
                value: ConfigNodeValue::ProfileBashrc(vec![
                    PathBuf::from("foo/profile.bashrc/a.sh"),
                    PathBuf::from("foo/profile.bashrc/b.sh"),
                ]),
            },
            ConfigNode {
                sources: vec![PathBuf::from("foo/package.bashrc/bar")],
                value: ConfigNodeValue::PackageBashrcs(vec![
                    PackageBashrc {
                        atom: ">=sys-lib/test-1".parse()?,
                        paths: vec![
                            PathBuf::from("foo/bashrc/test.sh"),
                            PathBuf::from("foo/bashrc/another.sh"),
                        ],
                    },
                    PackageBashrc {
                        atom: "sys-lib/none".parse()?,
                        paths: vec![PathBuf::from("foo/bashrc/other.sh")],
                    },
                ]),
            },
            ConfigNode {
                sources: vec![PathBuf::from("bar/profile.bashrc/c.sh")],
                value: ConfigNodeValue::ProfileBashrc(vec![PathBuf::from(
                    "bar/profile.bashrc/c.sh",
                )]),
            },
        ])]);

        let bashrcs = bundle.package_bashrcs(&PackageRef {
            package_name: "sys-lib/test",
            version: &"1".parse()?,
            slot: Some(Slot {
                main: "0",
                sub: "0",
            }),
            use_map: None,
            readiness: None,
        });

        assert_eq!(
            bashrcs.iter().map(|a| a.as_path()).collect::<Vec<_>>(),
            vec![
                Path::new("foo/profile.bashrc/a.sh"),
                Path::new("foo/profile.bashrc/b.sh"),
                Path::new("foo/bashrc/test.sh"),
                Path::new("foo/bashrc/another.sh"),
                Path::new("bar/profile.bashrc/c.sh",),
            ],
        );

        assert_eq!(
            bundle.all_profile_bashrcs(),
            vec![
                Path::new("foo/profile.bashrc/a.sh"),
                Path::new("foo/profile.bashrc/b.sh"),
                Path::new("bar/profile.bashrc/c.sh",),
            ],
        );

        assert_eq!(
            bundle.all_package_bashrcs(),
            vec![
                Path::new("foo/bashrc/test.sh"),
                Path::new("foo/bashrc/another.sh"),
                Path::new("foo/bashrc/other.sh"),
            ],
        );

        Ok(())
    }
}
