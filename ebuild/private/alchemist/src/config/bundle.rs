// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::{HashMap, HashSet},
    iter,
};

use itertools::Itertools;

use crate::{
    data::{IUseMap, UseMap, Vars},
    dependency::{
        package::{PackageRef, ThinPackageRef},
        Predicate,
    },
    version::Version,
};

use super::{
    ConfigNode, ConfigNodeValue, ConfigSource, PackageMaskKind, ProvidedPackage, UseUpdateKind,
};

struct BuiltinIncrementalVariable {
    name: &'static str,
    defaults: &'static str,
}

// A list of profile variables treated as incremental by default.
// https://projects.gentoo.org/pms/8/pms.html#x1-560005.3.1
const BUILTIN_INCREMENTAL_VARIABLES_EXCEPT_USE: &[BuiltinIncrementalVariable] = &[
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
        defaults: "",
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
        if let Some(token) = token.strip_prefix("-") {
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
    pub fn from_sources<S: ConfigSource, I: IntoIterator<Item = S>>(sources: I) -> Self {
        let mut env = Vars::new();
        let nodes = sources
            .into_iter()
            .flat_map(|source| source.evaluate_configs(&mut env))
            .collect();
        Self::new(env, nodes)
    }

    pub fn new(mut env: Vars, nodes: Vec<ConfigNode>) -> Self {
        // Compute incremental variables that are not specific to packages
        // (i.e. all except USE).
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

    /// Returns variables defined by underlying sources.
    ///
    /// Incremental variables are already resolved. Use [`compute_use_map`] to
    /// compute USE flags for a package, instead of reading `USE` variable with
    /// this method, since `USE` flags can vary by packages and thus it makes
    /// little sense to compute "global USE flags".
    ///
    /// This is often called as "profile variables", even though they can be
    /// defined in non-profile sources such as make.conf.
    pub fn env(&self) -> &Vars {
        &self.env
    }

    /// Computes USE flags of a package.
    pub fn compute_use_map(
        &self,
        package_name: &str,
        version: &Version,
        stable: bool,
        ebuild_iuse_map: &IUseMap,
    ) -> UseMap {
        let package = &ThinPackageRef {
            package_name,
            version,
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

        UseMap::from_iter(effective_iuse_map.iter().map(|(name, _)| {
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
        package: &'a ThinPackageRef,
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
                .chain(config_uses)
                .chain(self.use_expand_values.iter().map(|s| &**s)),
        )
    }

    /// Compute the masked USE flags of a package.
    fn compute_use_masks<'a>(
        &'a self,
        package: &'a ThinPackageRef,
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

    /// Compute the values of all incremental variables, except USE whose value
    /// varies by package.
    ///
    /// This function is supposed to be called from the constructor and its
    /// result should be cached, thus this function does not take self.
    fn compute_general_incremental_variables<'a>(
        nodes: &Vec<ConfigNode>,
    ) -> HashMap<String, Vec<String>> {
        // Compute built-in incremental variables.
        let builtins: HashMap<String, Vec<String>> = BUILTIN_INCREMENTAL_VARIABLES_EXCEPT_USE
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

        builtins.into_iter().chain(expands.into_iter()).collect()
    }

    /// Compute the value of an incremental variable.
    ///
    /// This function must not be used to compute the value of USE as its value
    /// varies by package. It will panic if [name] is "USE", which should happen
    /// only when USE is contained in USE_EXPAND or USE_EXPAND_UNPREFIXED.
    ///
    /// This function is supposed to be called from the constructor and its
    /// result should be cached, thus this function does not take self.
    fn compute_general_incremental_variable<'a>(
        nodes: &'a Vec<ConfigNode>,
        name: &'a str,
        defaults: &'a str,
    ) -> impl Iterator<Item = &'a str> {
        if name == "USE" {
            panic!("USE_EXPAND/USE_EXPAND_UNPREFIXED must not contain USE");
        }
        merge_incremental_tokens(
            iter::once(defaults)
                .chain(nodes.iter().filter_map(move |node| match &node.value {
                    ConfigNodeValue::Vars(vars) => vars.get(name).map(|value| &**value),
                    _ => None,
                }))
                .flat_map(|s| s.split_ascii_whitespace()),
        )
    }
}
