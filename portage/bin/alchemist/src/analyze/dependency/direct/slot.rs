// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use itertools::Itertools;

use crate::{
    data::UseMap,
    dependency::{
        algorithm::{elide_use_conditions, simplify},
        package::{PackageDependency, PackageSlotDependency},
        CompositeDependency, Dependency,
    },
    resolver::PackageResolver,
};

/// Rewrites the dependency expression by evaluating any USE constraints and
/// then sets the best slot and sub-slot for each dependency expression that
/// enables the sub-slot rebuild (:=) operator. The returned value is suitable
/// for inserting into the binpkg's XPAK.
pub fn rewrite_subslot_deps(
    deps: PackageDependency,
    use_map: &UseMap,
    resolver: &PackageResolver,
) -> Result<String> {
    let deps = elide_use_conditions(deps, use_map).unwrap_or_default();

    let deps = simplify(deps);

    // Rewrite atoms.
    let deps = deps.try_map_tree_par(|dep| -> Result<PackageDependency> {
        match dep {
            Dependency::Leaf(mut atom) => {
                if let Some(slot) = atom.slot() {
                    if !slot.rebuild_on_slot_change() {
                        return Ok(Dependency::Leaf(atom));
                    }

                    // Skip provided packages since we don't have slot
                    // information for them.
                    if resolver.find_provided_packages(&atom).next().is_some() {
                        return Ok(Dependency::Leaf(atom));
                    }

                    // Update slot operator.
                    match resolver.find_best_package_dependency(use_map, &atom) {
                        Ok(result) => {
                            if let Some(package) = result {
                                let slot = PackageSlotDependency::new(
                                    Some((
                                        package.slot.main.clone(),
                                        Some(package.slot.sub.clone()),
                                    )),
                                    slot.rebuild_on_slot_change(),
                                );

                                atom.set_slot(Some(slot));

                                return Ok(Dependency::Leaf(atom));
                            } else {
                                // any-of branch that didn't match.
                                return Ok(Dependency::Leaf(atom));
                            }
                        }
                        Err(err) => {
                            return Err(err).with_context(|| format!("Error matching {}", atom));
                        }
                    }
                }

                // Non-slot operator atom.
                Ok(Dependency::Leaf(atom))
            }
            _ => Ok(dep),
        }
    })?;

    let deps = simplify(deps);

    // We want to drop the `( )` that AllOf prints.
    let deps: Vec<PackageDependency> = match deps {
        Dependency::Composite(composite) => match *composite {
            CompositeDependency::AllOf { children } => children,
            CompositeDependency::AnyOf { .. } => vec![Dependency::new_composite(*composite)],
            other => bail!(
                "Found an unexpected composite dependency: {}",
                Dependency::new_composite(other)
            ),
        },
        Dependency::Leaf(leaf) => vec![Dependency::Leaf(leaf)],
    };

    let expr = deps.into_iter().join(" ");

    Ok(expr)
}
