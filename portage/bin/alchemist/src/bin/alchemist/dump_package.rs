// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alchemist::analyze::dependency::direct::analyze_direct_dependencies;
use alchemist::bash::vars::BashValue;
use alchemist::dependency::package::PackageAtom;
use alchemist::ebuild::{MaybePackageDetails, PackageDetails, PackageReadiness};
use alchemist::resolver::select_best_version;
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
        let default = select_best_version(&packages).cloned();

        packages.sort_by(|a, b| a.as_basic_data().version.cmp(&b.as_basic_data().version));
        packages.reverse();

        println!("=======\t{}", atom);

        for (i, maybe_details) in packages.into_iter().enumerate() {
            if i > 0 {
                println!();
            }

            let basic_data = maybe_details.as_basic_data();
            let is_default = match &default {
                Some(default) => default.as_basic_data().ebuild_path == basic_data.ebuild_path,
                None => false,
            };

            println!("Path:\t\t{}", &basic_data.ebuild_path.to_string_lossy());
            println!("Package:\t{}", &basic_data.package_name);
            println!(
                "Version:\t{}{}",
                &basic_data.version,
                if is_default { " (Default)" } else { "" }
            );

            let details = match maybe_details {
                MaybePackageDetails::Ok(details) => details,
                MaybePackageDetails::Err(error) => {
                    println!("WARNING: Failed to load package: {:#}", error.error);
                    continue;
                }
            };

            println!("Slot:\t\t{}", &details.slot);
            println!("Stable:\t\t{:?}", details.stable);
            match &details.readiness {
                PackageReadiness::Ok => {
                    println!("Readiness:\t\tOK (not masked)");
                }
                PackageReadiness::Masked { reason } => {
                    println!("Readiness:\t\tMasked ({})", reason);
                }
            }
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

            match analyze_direct_dependencies(&details, cross_compile, &host.resolver, resolver) {
                Ok((deps, _expressions)) => {
                    dump_deps("BDEPEND", &deps.build_host);
                    dump_deps("IDEPEND", &deps.install_host);
                    dump_deps("DEPEND", &deps.build_target);
                    dump_deps("RDEPEND", &deps.run_target);
                    dump_deps("PDEPEND", &deps.post_target);
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
