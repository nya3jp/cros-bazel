// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::data::Vars;

use super::{
    makeconf::MakeConf,
    miscconf::{
        accept_keywords::load_accept_keywords_configs, mask::load_package_configs,
        provided::load_provided_packages_config, useflags::load_use_configs,
    },
    ConfigNode, ConfigSource,
};

pub struct SiteSettings {
    confs: Vec<MakeConf>,
    precomputed_nodes: Vec<ConfigNode>,
}

impl SiteSettings {
    pub fn load(root_dir: &Path) -> Result<Self> {
        let mut confs = Vec::<MakeConf>::new();

        for rel_path in ["etc/make.conf", "etc/portage/make.conf"] {
            let path = root_dir.join(rel_path);
            if path.exists() {
                confs.push(
                    MakeConf::load(&path, path.parent().unwrap(), true, true)
                        .with_context(|| format!("Failed to load {}", path.display()))?,
                );
            }
        }

        // Either of the two make.conf files must exist.
        if confs.is_empty() {
            bail!(
                "make.conf not found under {} (have you run setup_board?)",
                root_dir.display()
            );
        }

        let portage_dir = root_dir.join("etc/portage");
        let site_profile_dir = portage_dir.join("profile");
        let precomputed_nodes = [
            load_package_configs(&site_profile_dir)?,
            load_accept_keywords_configs(&site_profile_dir)?,
            load_use_configs(&site_profile_dir)?,
            load_provided_packages_config(&site_profile_dir)?,
            load_package_configs(&portage_dir)?,
            load_accept_keywords_configs(&portage_dir)?,
            load_use_configs(&portage_dir)?,
            load_provided_packages_config(&portage_dir)?,
        ]
        .concat();

        Ok(Self {
            confs,
            precomputed_nodes,
        })
    }
}

impl ConfigSource for SiteSettings {
    fn evaluate_configs(&self, env: &mut Vars) -> Vec<ConfigNode> {
        self.confs
            .iter()
            .flat_map(|conf| conf.evaluate_configs(env))
            .chain(self.precomputed_nodes.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use version::Version;

    use crate::{
        config::{
            ConfigNodeValue, PackageMaskKind, PackageMaskUpdate, ProvidedPackage, UseUpdate,
            UseUpdateFilter, UseUpdateKind,
        },
        dependency::package::PackageAtom,
        testutils::write_files,
    };

    use super::*;

    #[test]
    fn test_load() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(
            dir,
            [
                ("etc/make.conf", "USE=a"),
                ("etc/portage/make.conf", "USE=b"),
                ("etc/portage/profile/package.mask", "pkg/x"),
                ("etc/portage/profile/package.use", "pkg/x foo -bar baz"),
                ("etc/portage/profile/package.provided", "pkg/x-1.0.0"),
                ("etc/portage/package.mask", "pkg/x"),
                ("etc/portage/package.use", "pkg/x foo -bar baz"),
                ("etc/portage/package.provided", "pkg/x-1.0.0"),
            ],
        )?;

        let site_settings = SiteSettings::load(dir)?;
        let mut env = Vars::new();
        let nodes = site_settings.evaluate_configs(&mut env);

        assert_eq!(Vars::from_iter([("USE".to_owned(), "b".to_owned())]), env);
        assert_eq!(
            vec![
                ConfigNode {
                    sources: vec![dir.join("etc/make.conf")],
                    value: ConfigNodeValue::Vars(Vars::from_iter([(
                        "USE".to_owned(),
                        "a".to_owned()
                    )])),
                },
                ConfigNode {
                    sources: vec![dir.join("etc/portage/make.conf")],
                    value: ConfigNodeValue::Vars(Vars::from_iter([(
                        "USE".to_owned(),
                        "b".to_owned()
                    )])),
                },
                ConfigNode {
                    sources: vec![dir.join("etc/portage/profile/package.mask")],
                    value: ConfigNodeValue::PackageMasks(vec![PackageMaskUpdate {
                        kind: PackageMaskKind::Mask,
                        atom: PackageAtom::from_str("pkg/x").unwrap(),
                    }]),
                },
                ConfigNode {
                    sources: vec![dir.join("etc/portage/profile/package.use")],
                    value: ConfigNodeValue::Uses(vec![UseUpdate {
                        kind: UseUpdateKind::Set,
                        filter: UseUpdateFilter {
                            atom: Some(PackageAtom::from_str("pkg/x").unwrap()),
                            stable_only: false,
                        },
                        use_tokens: "foo -bar baz".to_owned(),
                    }]),
                },
                ConfigNode {
                    sources: vec![dir.join("etc/portage/profile/package.provided")],
                    value: ConfigNodeValue::ProvidedPackages(vec![ProvidedPackage {
                        package_name: "pkg/x".to_owned(),
                        version: Version::try_new("1.0.0").unwrap(),
                    }]),
                },
                ConfigNode {
                    sources: vec![dir.join("etc/portage/package.mask")],
                    value: ConfigNodeValue::PackageMasks(vec![PackageMaskUpdate {
                        kind: PackageMaskKind::Mask,
                        atom: PackageAtom::from_str("pkg/x").unwrap(),
                    }]),
                },
                ConfigNode {
                    sources: vec![dir.join("etc/portage/package.use")],
                    value: ConfigNodeValue::Uses(vec![UseUpdate {
                        kind: UseUpdateKind::Set,
                        filter: UseUpdateFilter {
                            atom: Some(PackageAtom::from_str("pkg/x").unwrap()),
                            stable_only: false,
                        },
                        use_tokens: "foo -bar baz".to_owned(),
                    }]),
                },
                ConfigNode {
                    sources: vec![dir.join("etc/portage/package.provided")],
                    value: ConfigNodeValue::ProvidedPackages(vec![ProvidedPackage {
                        package_name: "pkg/x".to_owned(),
                        version: Version::try_new("1.0.0").unwrap(),
                    }]),
                },
            ],
            nodes
        );
        Ok(())
    }
}
