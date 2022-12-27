// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alchemist::{dependency::package::PackageAtomDependency, resolver::PackageResolver};
use anyhow::Result;
use colored::Colorize;
use itertools::Itertools;

pub fn dump_package_main(
    resolver: &PackageResolver,
    atoms: Vec<PackageAtomDependency>,
) -> Result<()> {
    for atom in atoms {
        let details_list = resolver.find_packages(&atom)?;

        println!("=======\t{}", atom);

        for details in details_list {
            println!("Path:\t\t{}", &details.ebuild_path.to_string_lossy());
            println!("Package:\t{}", &details.package_name);
            println!("Version:\t{}", &details.version);
            println!("Slot:\t\t{}", &details.slot);
            println!("Stability:\t{:?}", details.stability);
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
            println!();
        }
    }
    Ok(())
}
