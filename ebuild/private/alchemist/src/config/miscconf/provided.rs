// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use std::{fs::read_to_string, path::Path};

use crate::{
    config::{ConfigNode, ConfigNodeValue, ProvidedPackage},
    version::Version,
};

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

#[cfg(test)]
mod tests {
    use crate::testutils::write_files;

    use super::*;

    #[test]
    fn test_load_provided_packages_config() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(
            dir,
            [(
                "package.provided",
                r#"
                    # this is a comment line
                    pkg/a-1.0.0
                    pkg/b-2.0.0
                "#,
            )],
        )?;

        let nodes = load_provided_packages_config(dir)?;
        assert_eq!(
            vec![ConfigNode {
                source: dir.join("package.provided"),
                value: ConfigNodeValue::ProvidedPackages(vec![
                    ProvidedPackage {
                        package_name: "pkg/a".to_owned(),
                        version: Version::try_new("1.0.0").unwrap(),
                    },
                    ProvidedPackage {
                        package_name: "pkg/b".to_owned(),
                        version: Version::try_new("2.0.0").unwrap(),
                    },
                ]),
            }],
            nodes
        );
        Ok(())
    }
}
