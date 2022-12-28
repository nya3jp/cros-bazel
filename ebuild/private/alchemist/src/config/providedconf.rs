// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use std::{fs::read_to_string, path::Path};

use crate::version::Version;

use super::{ConfigNode, ConfigNodeValue, ProvidedPackage};

pub fn load_provided_packages_config(dir: &Path) -> Result<Vec<ConfigNode>> {
    let source = dir.join("package.provided");

    if !source.exists() {
        return Ok(Vec::new());
    }

    let contents = read_to_string(&source)?;

    let mut packages = Vec::<ProvidedPackage>::new();

    for line in contents
        .split("\n")
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with("#"))
    {
        let (package_name, version) = Version::from_str_suffix(line)?;
        packages.push(ProvidedPackage {
            package_name: package_name.to_owned(),
            version,
        });
    }

    Ok(vec![ConfigNode {
        source,
        value: ConfigNodeValue::ProvidedPackages(packages),
    }])
}
