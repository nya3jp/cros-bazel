// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::fmt::Display;

use anyhow::{bail, Result};
use itertools::Itertools;

use crate::data::UseMap;

use super::{CompositeDependency, Dependency, DependencyMeta};

/// Elides USE conditions (`foo? ( ... )`) from a dependency expression by
/// assigning USE flag values.
pub fn elide_use_conditions<M: DependencyMeta>(
    deps: Dependency<M>,
    use_map: &UseMap,
) -> Option<Dependency<M>> {
    deps.flat_map_tree(|d| {
        match d {
            Dependency::Composite(composite) => {
                match *composite {
                    CompositeDependency::UseConditional {
                        name,
                        expect,
                        child,
                    } => {
                        // Assume that a USE flag is unset when it is not declared in IUSE.
                        // TODO: Check if this is a right behavior.
                        let value = *use_map.get(&name).unwrap_or(&false);
                        if value == expect {
                            Some(child)
                        } else {
                            None
                        }
                    }
                    other => Some(Dependency::Composite(Box::new(other))),
                }
            }
            other => Some(other),
        }
    })
}

/// Simplifies a dependency expression by eliding unnecessary items.
///
/// For example, if an any-of expression contains a constant true as a child,
/// it is simplified to a constant true.
pub fn simplify<M: DependencyMeta>(deps: Dependency<M>) -> Dependency<M> {
    deps.map_tree(|d| {
        match d {
            Dependency::Composite(composite) => {
                match *composite {
                    CompositeDependency::AllOf { children } => {
                        let children = children
                            .into_iter()
                            // Drop the constant true.
                            .filter(|d| !matches!(d.check_constant(), Some((true, _))))
                            // Merge nested all-of.
                            .flat_map(|d| match d {
                                Dependency::Composite(composite) => match *composite {
                                    CompositeDependency::AllOf { children } => children,
                                    other => vec![Dependency::Composite(Box::new(other))],
                                },
                                other => vec![other],
                            })
                            .collect_vec();
                        let first_constant_false = children
                            .iter()
                            .flat_map(|child| match child.check_constant() {
                                Some((false, reason)) => Some(reason),
                                _ => None,
                            })
                            .next();
                        if let Some(reason) = first_constant_false {
                            Dependency::new_constant(false, reason)
                        } else if children.len() == 1 {
                            children.into_iter().next().unwrap()
                        } else {
                            Dependency::Composite(Box::new(CompositeDependency::AllOf { children }))
                        }
                    }
                    CompositeDependency::AnyOf { children } => {
                        let mut false_constants = vec![];
                        let mut others = vec![];

                        for child in children {
                            match child.check_constant() {
                                Some((true, _)) => return child,
                                Some((false, _)) => false_constants.push(child),
                                None => others.push(child),
                            }
                        }

                        if others.len() == 1 {
                            others.into_iter().next().unwrap()
                        } else if others.len() > 1 {
                            Dependency::Composite(Box::new(CompositeDependency::AnyOf {
                                children: others,
                            }))
                        } else if false_constants.len() == 1 {
                            false_constants.into_iter().next().unwrap()
                        } else if false_constants.len() > 1 {
                            Dependency::Composite(Box::new(CompositeDependency::AnyOf {
                                children: false_constants,
                            }))
                        } else {
                            Dependency::Composite(Box::new(CompositeDependency::AnyOf {
                                children: vec![],
                            }))
                        }
                    }
                    other => Dependency::Composite(Box::new(other)),
                }
            }
            other => other,
        }
    })
}

/// Converts a dependency expression to a list of leaf dependencies if it is
/// a leaf dependency or an "all-of" of leaf dependencies.
pub fn parse_simplified_dependency<M: DependencyMeta>(deps: Dependency<M>) -> Result<Vec<M::Leaf>>
where
    M::Leaf: Clone + Display + Eq + Ord,
{
    match deps {
        Dependency::Leaf(atom) => Ok(vec![atom]),
        Dependency::Composite(composite) => match *composite {
            CompositeDependency::AllOf { children } => {
                let atoms = children
                    .into_iter()
                    .map(|child| match child {
                        Dependency::Leaf(atom) => Ok(atom),
                        _ => bail!(
                            "Found a non-atom dependency after simplification: {}",
                            child
                        ),
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(atoms.into_iter().sorted().dedup().collect())
            }
            CompositeDependency::Constant {
                value: false,
                reason,
            } => {
                bail!("Unsatisfiable dependency: {}", reason);
            }
            other => bail!(
                "Found a non-atom dependency after simplification: {}",
                Dependency::new_composite(other)
            ),
        },
    }
}
