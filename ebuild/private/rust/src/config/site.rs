// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::path::Path;

use anyhow::{Context, Result};

use crate::data::Vars;

use super::{
    makeconf::MakeConf, pkgconf::load_package_configs, providedconf::load_provided_packages_config,
    useconf::load_use_configs, ConfigNode, ConfigSource,
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
            confs.push(
                MakeConf::load(&path, path.parent().unwrap(), true, true)
                    .with_context(|| format!("Loading {}", path.to_string_lossy()))?,
            );
        }

        let portage_dir = root_dir.join("etc/portage");
        let site_profile_dir = portage_dir.join("profile");
        let precomputed_nodes = [
            load_package_configs(&site_profile_dir)?,
            load_use_configs(&site_profile_dir)?,
            load_provided_packages_config(&site_profile_dir)?,
            load_package_configs(&portage_dir)?,
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
            .chain(self.precomputed_nodes.clone().into_iter())
            .collect()
    }
}
