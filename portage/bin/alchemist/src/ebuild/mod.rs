// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

pub mod metadata;

use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use version::Version;

use crate::{
    bash::vars::BashVars,
    config::bundle::{ConfigBundle, IsPackageAcceptedResult},
    data::{IUseMap, Slot, UseMap},
    dependency::package::{PackageRef, ThinPackageRef},
};

use self::metadata::CachedEBuildEvaluator;

/// Parses IUSE defined by ebuild/eclasses and returns as an [IUseMap].
fn parse_iuse_map(vars: &BashVars) -> Result<IUseMap> {
    Ok(vars
        .get_scalar_or_default("IUSE")?
        .split_ascii_whitespace()
        .map(|token| {
            if let Some(name) = token.strip_prefix('+') {
                return (name, true);
            }
            if let Some(name) = token.strip_prefix('-') {
                return (name, false);
            }
            (token, false)
        })
        .map(|(name, value)| (name.to_owned(), value))
        .collect())
}

type PackageResult = Result<PackageDetails, PackageError>;

/// Holds the error that occurred when processing the ebuild.
#[derive(Clone, Debug)]
pub struct PackageError {
    pub repo_name: String,
    pub package_name: String,
    pub ebuild: PathBuf,
    pub ebuild_name: String,
    pub version: Version,
    pub masked: Option<bool>,
    pub error: String,
}

#[derive(Clone, Debug)]
pub struct PackageDetails {
    pub repo_name: String,
    pub package_name: String,
    pub version: Version,
    pub vars: BashVars,
    pub slot: Slot,
    pub use_map: UseMap,
    pub accepted: bool,
    pub stable: bool,
    pub masked: bool,
    pub ebuild_path: PathBuf,
    pub inherited: HashSet<String>,
    pub inherit_paths: Vec<PathBuf>,
    pub direct_build_target: Option<String>,
}

impl PackageDetails {
    /// Converts this PackageDetails to a PackageRef that can be passed to
    /// dependency predicates.
    pub fn as_package_ref(&self) -> PackageRef {
        PackageRef {
            package_name: &self.package_name,
            version: &self.version,
            slot: Slot {
                main: self.slot.main.as_str(),
                sub: self.slot.sub.as_str(),
            },
            use_map: &self.use_map,
        }
    }

    pub fn as_thin_package_ref(&self) -> ThinPackageRef {
        ThinPackageRef {
            package_name: &self.package_name,
            version: &self.version,
            slot: Slot {
                main: self.slot.main.as_str(),
                sub: self.slot.sub.as_str(),
            },
        }
    }

    /// EAPI is technically a string, but working with an integer is easier.
    fn eapi(&self) -> Result<i32> {
        let eapi = self.vars.get_scalar("EAPI")?;
        eapi.parse::<i32>().with_context(|| format!("EAPI: {eapi}"))
    }

    pub fn supports_bdepend(&self) -> bool {
        let eapi = match self.eapi() {
            Ok(val) => val,
            Err(_) => return false,
        };

        eapi >= 7
    }

    /// Returns true if the package modifies ROOT or / using pkg hooks when
    /// installed. i.e., does it have pkg_preinst or pkg_postinst defined.
    ///
    /// See https://projects.gentoo.org/pms/8/pms.html#x1-11500011.3
    ///
    /// TODO: It's possible for `pkg_setup` to both modify ROOT and /, but
    /// in practice most `pkg_setup` functions don't. Instead they just setup
    /// the environment for the ebuidl to execute. Ideally a `src_setup`
    /// function could be used instead. The one case we have today that modifies
    /// ROOT and / is user/group creation. These packages need to be migrated to
    /// use the acct-{user,group} packages. Omitting `pkg_setup` from this list
    /// hasn't caused any problems yet.
    ///
    /// TODO: We also don't check for the cros .bashrc hooks. We don't have
    /// consume these at analysis time, so we don't actually know if they are
    /// present. The .bashrc files also apply cross-repo which makes things
    /// complicated. After auditing the scripts, it looks like imagemagick
    /// might cause problems. We can add an exclude list into this function
    /// if necessary while we clean up the hooks.
    pub fn has_hooks(&self) -> bool {
        let phases = self
            .vars
            .get_indexed_array("__xbuild_defined_phases")
            .expect("__xbuild_defined_phases to exist");

        phases.iter().any(|x| x == "preinst" || x == "postinst") || self.inherited.contains("user")
    }
}

#[derive(Debug)]
pub struct PackageLoader {
    evaluator: Arc<CachedEBuildEvaluator>,
    config: Arc<ConfigBundle>,
    force_accept_9999_ebuilds: bool,
    version_9999: Version,
}

impl PackageLoader {
    pub fn new(
        evaluator: Arc<CachedEBuildEvaluator>,
        config: Arc<ConfigBundle>,
        force_accept_9999_ebuilds: bool,
    ) -> Self {
        Self {
            evaluator,
            config,
            force_accept_9999_ebuilds,
            version_9999: Version::try_new("9999").unwrap(),
        }
    }

    pub fn load_package(&self, ebuild_path: &Path) -> Result<PackageResult> {
        // Drive the ebuild to read its metadata.
        let metadata = self.evaluator.evaluate_metadata(ebuild_path)?;

        // Compute additional information needed to fill in PackageDetails.
        let package_name = format!(
            "{}/{}",
            metadata.path_info.category_name, metadata.path_info.package_short_name,
        );

        let vars = match &metadata.vars {
            Ok(vars) => vars,
            Err(e) => {
                return Ok(PackageResult::Err(PackageError {
                    repo_name: metadata.repo_name.clone(),
                    package_name,
                    ebuild: ebuild_path.to_owned(),
                    ebuild_name: ebuild_path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                    version: metadata.path_info.version.clone(),
                    masked: None,
                    error: e.to_string(),
                }))
            }
        };

        let slot = Slot::<String>::new(vars.get_scalar("SLOT")?);

        let package = ThinPackageRef {
            package_name: package_name.as_str(),
            version: &metadata.path_info.version,
            slot: Slot {
                main: &slot.main,
                sub: &slot.sub,
            },
        };

        let raw_inherited = vars.get_scalar_or_default("INHERITED")?;
        let inherited: HashSet<String> = raw_inherited
            .split_ascii_whitespace()
            .map(|s| s.to_owned())
            .collect();

        let raw_inherit_paths = vars.get_indexed_array("INHERIT_PATHS")?;
        let inherit_paths: Vec<PathBuf> =
            raw_inherit_paths.into_iter().map(PathBuf::from).collect();

        let (accepted, stable) = match self.config.is_package_accepted(&vars, &package)? {
            IsPackageAcceptedResult::Unaccepted => {
                if self.force_accept_9999_ebuilds {
                    let accepted = inherited.contains("cros-workon")
                        && metadata.path_info.version == self.version_9999
                        && match vars.get_scalar("CROS_WORKON_MANUAL_UPREV") {
                            Ok(value) => value != "1",
                            Err(_) => false,
                        };
                    (accepted, false)
                } else {
                    (false, false)
                }
            }
            IsPackageAcceptedResult::Accepted(stable) => (true, stable),
        };

        let iuse_map = parse_iuse_map(&vars)?;
        let use_map = self.config.compute_use_map(
            &package_name,
            &metadata.path_info.version,
            stable,
            &slot,
            &iuse_map,
        );

        let masked = !accepted || self.config.is_package_masked(&package);

        let direct_build_target = vars.maybe_get_scalar("METALLURGY_TARGET")?.map(|s| {
            if s.starts_with("@") {
                s.to_string()
            } else {
                // eg. //bazel:foo -> @@//bazel:foo
                format!("@@{s}")
            }
        });

        Ok(PackageResult::Ok(PackageDetails {
            repo_name: metadata.repo_name.clone(),
            package_name,
            version: metadata.path_info.version.clone(),
            vars: vars.clone(),
            slot,
            use_map,
            accepted,
            stable,
            masked,
            inherited,
            inherit_paths,
            ebuild_path: ebuild_path.to_owned(),
            direct_build_target,
        }))
    }
}

type CachedPackageResult = std::result::Result<Arc<PackageDetails>, Arc<PackageError>>;

/// Wraps PackageLoader to cache results.
#[derive(Debug)]
pub struct CachedPackageLoader {
    loader: PackageLoader,
    cache: Mutex<HashMap<PathBuf, Arc<OnceCell<CachedPackageResult>>>>,
}

impl CachedPackageLoader {
    pub fn new(loader: PackageLoader) -> Self {
        Self {
            loader,
            cache: Default::default(),
        }
    }

    pub fn load_package(&self, ebuild_path: &Path) -> Result<CachedPackageResult> {
        let once_cell = {
            let mut cache_guard = self.cache.lock().unwrap();
            cache_guard
                .entry(ebuild_path.to_owned())
                .or_default()
                .clone()
        };
        let details = once_cell.get_or_try_init(|| -> Result<CachedPackageResult> {
            match self.loader.load_package(ebuild_path)? {
                PackageResult::Ok(details) => {
                    Result::Ok(CachedPackageResult::Ok(Arc::new(details)))
                }
                PackageResult::Err(err) => Result::Ok(CachedPackageResult::Err(Arc::new(err))),
            }
        })?;
        Ok(details.clone())
    }
}
