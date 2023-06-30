// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use std::{fs::read_to_string, path::Path};
use version::Version;

use crate::config::{ConfigNode, ConfigNodeValue, ProvidedPackage};

pub fn load_provided_packages_config(dir: &Path) -> Result<Vec<ConfigNode>> {
    let source = dir.join("package.provided");

    if !source.exists() {
        return Ok(Vec::new());
    }

    let contents = read_to_string(&source)?;

    let mut packages = Vec::<ProvidedPackage>::new();

    for (lineno, line) in contents
        .split('\n')
        .map(|line| line.trim())
        .enumerate()
        .filter(|(_, line)| !line.is_empty() && !line.starts_with('#'))
    {
        let (package_name, version) = Version::from_str_suffix(line).with_context(|| {
            format!(
                "Failed to load {}: syntax error at line {}",
                source.display(),
                lineno + 1
            )
        })?;
        packages.push(ProvidedPackage {
            package_name: package_name.to_owned(),
            version,
        });
    }

    Ok(vec![ConfigNode {
        sources: vec![source],
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
                sources: vec![dir.join("package.provided")],
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
