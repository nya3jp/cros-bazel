// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alchemist::dependency::package::PackageAtom;
use alchemist::ebuild::PackageDetails;
use alchemist::{analyze::dependency::analyze_dependencies, bash::vars::BashValue};
use anyhow::{Context, Result};
use colored::Colorize;
use itertools::Itertools;
use std::sync::Arc;

use crate::alchemist::TargetData;

#[derive(clap::Args, Clone, Debug)]
pub struct Args {
    /// Additionally dump the environment for the package.
    #[arg(short = 'e', long)]
    env: bool,

    /// Package names.
    packages: Vec<String>,
}

fn dump_deps(dep_type: &str, deps: &Vec<Arc<PackageDetails>>) {
    println!("{dep_type}:");
    for dep in deps {
        println!(
            "  {}-{}::{}",
            dep.as_basic_data().package_name,
            dep.as_basic_data().version,
            dep.as_basic_data().repo_name
        );
    }
}

pub fn dump_package_main(host: &TargetData, target: Option<&TargetData>, args: Args) -> Result<()> {
    let atoms = args
        .packages
        .iter()
        .map(|raw| raw.parse::<PackageAtom>())
        .collect::<Result<Vec<_>>>()?;

    let resolver = &target.unwrap_or(host).resolver;

    let cross_compile = if let Some(target) = target {
        let cbuild = host
            .config
            .env()
            .get("CHOST")
            .context("host is missing CHOST")?;
        let chost = target
            .config
            .env()
            .get("CHOST")
            .context("target is missing CHOST")?;
        cbuild != chost
    } else {
        false
    };

    for atom in atoms {
        let mut packages = resolver.find_packages(&atom)?;
        let default = resolver.find_best_package_in(&packages)?;

        packages.sort_by(|a, b| a.as_basic_data().version.cmp(&b.as_basic_data().version));
        packages.reverse();

        println!("=======\t{}", atom);

        for (i, details) in packages.into_iter().enumerate() {
            if i > 0 {
                println!();
            }

            let is_default = match &default {
                Some(default) => {
                    default.as_basic_data().ebuild_path == details.as_basic_data().ebuild_path
                }
                None => false,
            };
            println!(
                "Path:\t\t{}",
                &details.as_basic_data().ebuild_path.to_string_lossy()
            );
            println!("Package:\t{}", &details.as_basic_data().package_name);
            println!(
                "Version:\t{}{}",
                &details.as_basic_data().version,
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

            match analyze_dependencies(&details, cross_compile, Some(&host.resolver), resolver) {
                Ok(deps) => {
                    dump_deps("BDEPEND", &deps.build_host_deps);
                    dump_deps("IDEPEND", &deps.install_host_deps);
                    dump_deps("DEPEND", &deps.build_deps);
                    dump_deps("RDEPEND", &deps.runtime_deps);
                    dump_deps("PDEPEND", &deps.post_deps);
                }
                Err(err) => {
                    println!("WARNING: Failed to analyze dependencies: {:#}", err);
                }
            }

            if args.env {
                println!("Env: ");
                let map = details.metadata.vars.hash_map();
                for key in map.keys().sorted() {
                    println!(
                        "  \"{}\": {}",
                        key,
                        match map.get(key).unwrap() {
                            BashValue::Scalar(val) => format!("\"{}\"", val),
                            BashValue::IndexedArray(vec) => format!("{:#?}", vec)
                                .lines()
                                .map(|line| format!("  {line}"))
                                .join("\n"),
                            BashValue::AssociativeArray(map) => format!("{:#?}", map)
                                .lines()
                                .map(|line| format!("  {line}"))
                                .join("\n"),
                        }
                    );
                }
            }
        }
    }
    Ok(())
}
