// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alchemist::analyze::dependency::analyze_dependencies;
use alchemist::ebuild::PackageDetails;
use alchemist::{dependency::package::PackageAtom, resolver::PackageResolver};
use anyhow::Result;
use colored::Colorize;
use itertools::Itertools;
use std::sync::Arc;

fn dump_deps(dep_type: &str, deps: &Vec<Arc<PackageDetails>>) {
    println!("{dep_type}:");
    for dep in deps {
        println!("  {}-{}", dep.package_name, dep.version);
    }
}

pub fn dump_package_main(resolver: &PackageResolver, atoms: Vec<PackageAtom>) -> Result<()> {
    for atom in atoms {
        let mut packages = resolver.find_packages(&atom)?;
        let default = resolver.find_best_package_in(&packages)?;

        packages.sort_by(|a, b| a.version.cmp(&b.version));
        packages.reverse();

        println!("=======\t{}", atom);

        for (i, details) in packages.into_iter().enumerate() {
            if i > 0 {
                println!();
            }

            let is_default = match &default {
                Some(default) => default.ebuild_path == details.ebuild_path,
                None => false,
            };
            println!("Path:\t\t{}", &details.ebuild_path.to_string_lossy());
            println!("Package:\t{}", &details.package_name);
            println!(
                "Version:\t{}{}",
                &details.version,
                if is_default { " (Default)" } else { "" }
            );
            println!("Slot:\t\t{}", &details.slot);
            println!("Accepted:\t{:?}", details.accepted);
            println!("Stable:\t\t{:?}", details.stable);
            println!("Masked:\t\t{}", details.masked);
            println!(
                "USE:\t\t{}",
                details
                    .use_map
                    .iter()
                    .sorted()
                    .map(|(name, value)| {
                        let label = format!("{}{}", if *value { "+" } else { "-" }, name);
                        label.color(if *value { "red" } else { "blue" })
                    })
                    .join(" ")
            );

            let deps = analyze_dependencies(&details, None, resolver)?;
            dump_deps("BDEPEND", &deps.build_host_deps);
            dump_deps("IDEPEND", &deps.install_host_deps);
            dump_deps("DEPEND", &deps.build_deps);
            dump_deps("RDEPEND", &deps.runtime_deps);
            dump_deps("PDEPEND", &deps.post_deps);
        }
    }
    Ok(())
}
