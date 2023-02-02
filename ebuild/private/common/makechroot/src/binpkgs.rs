// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use std::path::Path;

pub fn copy_binary_packages<P: AsRef<Path>>(
    packages_dir: &Path,
    package_paths: &[P],
) -> Result<Vec<String>> {
    const BINARY_EXT: &str = ".tbz2";

    let mut atoms = Vec::new();

    for package_path in package_paths {
        let xp = binarypackage::BinaryPackage::new(package_path)?.xpak()?;

        let category = std::str::from_utf8(&xp["CATEGORY"])?.trim();
        let pf = std::str::from_utf8(&xp["PF"])?.trim();

        let category_dir = packages_dir.join(category);
        std::fs::create_dir_all(&category_dir)?;

        let copy_path = category_dir.join(pf.to_owned() + BINARY_EXT);
        std::fs::copy(package_path, copy_path)?;

        atoms.push(format!("={}/{}", category, pf));
    }
    Ok(atoms)
}
