// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use itertools::Itertools;

use crate::data::UseMap;

use super::{CompositeDependency, Dependency};

/// Elides USE conditions (`foo? ( ... )`) from a dependency expression by
/// assigning USE flag values.
pub fn elide_use_conditions<L>(deps: Dependency<L>, use_map: &UseMap) -> Dependency<L> {
    deps.map_tree(|d| {
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
                            child
                        } else {
                            // TODO: Remove the expression rather than replacing it with
                            // the constant true.
                            // The current behavior does not match Portage's.
                            // For example, "|| ( foo? ( a/b ) )" should be considered
                            // unsatisfiable when foo is unset.
                            Dependency::new_constant(true)
                        }
                    }
                    other => Dependency::Composite(Box::new(other)),
                }
            }
            other => other,
        }
    })
}

/// Simplifies a dependency expression by eliding unnecessary items.
///
/// For example, if an any-of expression contains a constant true as a child,
/// it is simplified to a constant true.
pub fn simplify<L>(deps: Dependency<L>) -> Dependency<L> {
    deps.map_tree(|d| {
        match d {
            Dependency::Composite(composite) => {
                match *composite {
                    CompositeDependency::AllOf { children } => {
                        let children = children
                            .into_iter()
                            // Drop the constant true.
                            .filter(|d| d.is_constant() != Some(true))
                            // Merge nested all-of.
                            .flat_map(|d| match d {
                                Dependency::Composite(composite) => match *composite {
                                    CompositeDependency::AllOf { children } => children,
                                    other => vec![Dependency::Composite(Box::new(other))],
                                },
                                other => vec![other],
                            })
                            .collect_vec();
                        if children.iter().any(|d| d.is_constant() == Some(false)) {
                            Dependency::new_constant(false)
                        } else if children.len() == 1 {
                            children.into_iter().next().unwrap()
                        } else {
                            Dependency::Composite(Box::new(CompositeDependency::AllOf { children }))
                        }
                    }
                    CompositeDependency::AnyOf { children } => {
                        let children = children
                            .into_iter()
                            // Drop the constant false.
                            .filter(|d| d.is_constant() != Some(false))
                            .collect_vec();
                        if children.iter().any(|d| d.is_constant() == Some(true)) {
                            Dependency::new_constant(true)
                        } else if children.len() == 1 {
                            children.into_iter().next().unwrap()
                        } else {
                            Dependency::Composite(Box::new(CompositeDependency::AnyOf { children }))
                        }
                    }
                    other => Dependency::Composite(Box::new(other)),
                }
            }
            other => other,
        }
    })
}
