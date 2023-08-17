// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::HashMap,
    fs::{create_dir_all, File},
    io::Write,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    sync::Arc,
};

use alchemist::{
    analyze::{restrict::analyze_restricts, source::PackageLocalSource},
    config::ProvidedPackage,
    dependency::restrict::RestrictAtom,
    ebuild::{PackageDetails, PackageError},
    fakechroot::PathTranslator,
    repository::RepositorySet,
};
use anyhow::{anyhow, Result};
use itertools::Itertools;
use lazy_static::lazy_static;
use rayon::prelude::*;
use serde::Serialize;
use tera::Tera;
use tracing::instrument;

use crate::generate_repo::common::{
    package_details_to_target_path, repository_set_to_target_path, DistFileEntry, Package,
    AUTOGENERATE_NOTICE, PRIMORDIAL_PACKAGES,
};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera: Tera = Default::default();
        tera.add_raw_template(
            "package.BUILD.bazel",
            include_str!("templates/package.BUILD.bazel"),
        )
        .unwrap();
        tera
    };
}

#[derive(Serialize)]
pub struct EBuildEntry {
    ebuild_name: String,
    basename: String,
    overlay: String,
    category: String,
    package_name: String,
    version: String,
    slot: String,
    sources: Vec<String>,
    git_trees: Vec<String>,
    dists: Vec<DistFileEntry>,
    eclasses: Vec<String>,
    provided_host_build_deps: Vec<String>,
    host_build_deps: Vec<String>,
    host_install_deps: Vec<String>,
    target_build_deps: Vec<String>,
    provided_runtime_deps: Vec<String>,
    runtime_deps: Vec<String>,
    install_set: Vec<String>,
    allow_network_access: bool,
    uses: String,
    sdk: String,
    direct_build_target: Option<String>,
    has_hooks: bool,
}

/// Specifies the config used to generate host packages.
#[derive(Debug, Copy, Clone)]
pub struct PackageHostConfig<'a> {
    /// The host repository set that contains all the portage config.
    pub repo_set: &'a RepositorySet,

    /// Prefix to append to the package paths. It also defines the path to the
    /// SDK to use to build these packages.
    ///
    /// i.e., Passing "stage2/host" will result in a package BUILD file getting
    /// generated at //internal/packages/stage2/host/sys-libs/glibc/BUILD.bazel.
    ///
    /// The ebuild targets will use the `//internal/sdk/stage2/host` SDK.
    pub prefix: &'a str,

    /// The packages provided by the SDK. This allows us to skip re-installing
    /// these packages into the package's "deps" layer and also avoids circular
    /// dependencies.
    pub sdk_provided_packages: &'a [ProvidedPackage],
}

/// Specifies the config used to generate packages for the target board.
#[derive(Debug, Copy, Clone)]
pub struct PackageTargetConfig<'a> {
    /// The target repository set that contains all the portage config.
    pub repo_set: &'a RepositorySet,

    /// The board name used to derive the ROOT parameter.
    /// i.e., /build/${BOARD}
    pub board: &'a str,

    /// Prefix to append to the package paths. It also defines the path to
    // the SDK to use to build these packages.
    ///
    /// i.e., Passing "stage2/target/board" will result in a package BUILD
    /// file getting generated at
    /// `//internal/packages/stage2/target/board/sys-libs/glibc/BUILD.bazel`.
    ///
    /// The ebuild targets will use the `//internal/sdk/stage2/target/board`
    /// SDK.
    pub prefix: &'a str,
}

/// Specifies the type of packages to generate.
pub enum PackageType<'a> {
    /// Packages will be generated to compile against the host's SYSROOT.
    ///
    /// i.e., ROOT=/
    Host(PackageHostConfig<'a>),

    /// Packages will be generated to compile in their own SYSROOT.
    ///
    /// i.e., ROOT=/build/$BOARD
    ///
    /// This is called a cross-root build because we are using the host tools
    /// defined in / to build the packages in a different SYSROOT. A
    /// cross-compile build is specialization of a cross-root build. It's
    /// defined as a build where CBUILD != CHOST. Since we don't specify the
    /// CBUILD or CHOST in this structure we don't know if its a cross-compile.
    CrossRoot {
        /// The host packages to use to satisfy BDEPEND / IDEPEND dependencies
        /// for the target packages.
        host: Option<PackageHostConfig<'a>>,
        /// The target to generate packages for.
        target: PackageTargetConfig<'a>,
    },
}

/// Splits the provided `packages` into two lists:
/// 1) `PackageDetails` that don't match the specified `provided` list.
/// 2) `PackageDetails` that do match the `provided` list.
fn partition_provided<'a>(
    packages: impl IntoIterator<Item = &'a Arc<PackageDetails>>,
    provided: &'a [ProvidedPackage],
) -> (Vec<&Arc<PackageDetails>>, Vec<&Arc<PackageDetails>>) {
    let (build_host_deps, provided_host_deps): (Vec<_>, Vec<_>) =
        packages.into_iter().partition(|package| {
            !provided.iter().any(|provided| {
                provided.package_name == package.package_name && provided.version == package.version
            })
        });

    (build_host_deps, provided_host_deps)
}

/// Converts the `PackageDetails` items into bazel paths using the provided
/// prefix.
fn format_dependencies<'a>(
    prefix: &str,
    deps: impl IntoIterator<Item = &'a Arc<PackageDetails>>,
) -> Result<Vec<String>> {
    let targets = deps
        .into_iter()
        .map(|details| package_details_to_target_path(details, prefix))
        .collect::<Vec<_>>();
    Ok(targets.into_iter().sorted().dedup().collect())
}

impl EBuildEntry {
    pub fn try_new(target: &PackageType, package: &Package) -> Result<Self> {
        let ebuild_name = package
            .details
            .ebuild_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let basename = ebuild_name
            .rsplit_once('.')
            .ok_or_else(|| anyhow!("No file extension"))?
            .0
            .to_owned();
        let (category, package_name) = package
            .details
            .package_name
            .split_once('/')
            .expect("Package name must contain a /");

        let version = package.details.version.to_string();

        let sources = package
            .sources
            .local_sources
            .iter()
            .map(|source| match source {
                PackageLocalSource::BazelTarget(target) => target.clone(),
                PackageLocalSource::Src(src) => {
                    format!("//internal/sources/{}:__tarballs__", src.to_string_lossy())
                }
                PackageLocalSource::Chrome(version) => {
                    format!("@portage_deps//:chrome-{version}_src")
                }
                PackageLocalSource::Chromite => "@chromite//:src".to_string(),
            })
            .collect();

        let git_trees = package
            .sources
            .repo_sources
            .iter()
            .map(|source| format!("@portage_deps//:{}_src", source.name.to_owned()))
            .collect();

        let dists = package
            .sources
            .dist_sources
            .iter()
            .map(DistFileEntry::try_new)
            .collect::<Result<_>>()?;

        let eclasses = package
            .details
            .inherit_paths
            .iter()
            .map(|path| {
                let eclass = path.file_stem().unwrap().to_string_lossy();

                let repo_set = match &target {
                    PackageType::Host(host) => host.repo_set,
                    PackageType::CrossRoot { target, .. } => target.repo_set,
                };
                let repo = repo_set.get_repo_by_path(path).unwrap().name();

                format!("//internal/overlays/{}/eclass:{}", repo, eclass)
            })
            .collect();

        let (host_build_deps, provided_host_build_deps) = match &target {
            // When building host packages we need to ensure DEPEND packages
            // are present on the host.
            PackageType::Host(host) => {
                let (host_build_deps, provided_host_build_deps) = partition_provided(
                    package
                        .build_host_deps
                        .iter()
                        .chain(package.dependencies.build_deps.iter())
                        .unique_by(|details| &details.ebuild_path),
                    host.sdk_provided_packages,
                );

                let mut host_build_deps =
                    format_dependencies(host.prefix, host_build_deps.into_iter())?;
                host_build_deps.sort();

                (host_build_deps, provided_host_build_deps)
            }
            PackageType::CrossRoot { host, .. } => {
                // Stage 1 packages don't have a host since we don't know
                // what's contained in the stage1 SDK.
                if let Some(host) = host {
                    let (host_build_deps, provided_host_build_deps) = partition_provided(
                        package.build_host_deps.iter(),
                        host.sdk_provided_packages,
                    );

                    let mut host_build_deps =
                        format_dependencies(host.prefix, host_build_deps.into_iter())?;
                    host_build_deps.sort();

                    (host_build_deps, provided_host_build_deps)
                } else {
                    (Vec::new(), Vec::new())
                }
            }
        };

        // Convert into a purely human readable form. We just use this list to aid in
        // documentation in case someone is debugging a dependency problem.
        let provided_host_build_deps = provided_host_build_deps
            .into_iter()
            .map(|details| format!("{}-{}", details.package_name, details.version))
            .sorted()
            .collect();

        let target_build_deps = match &target {
            // Host DEPENDs are handled above with the host_build_deps
            PackageType::Host { .. } => Vec::new(),
            PackageType::CrossRoot { target, .. } => {
                // TODO: Add support for stripping the Board SDK's packages.
                format_dependencies(target.prefix, package.dependencies.build_deps.iter())?
            }
        };

        let (runtime_deps, provided_runtime_deps) = match &target {
            PackageType::Host(host) => {
                let (runtime_deps, provided_runtime_deps) = partition_provided(
                    package.dependencies.runtime_deps.iter(),
                    host.sdk_provided_packages,
                );

                let runtime_deps = format_dependencies(host.prefix, runtime_deps.into_iter())?;

                let provided_runtime_deps = provided_runtime_deps
                    .iter()
                    .map(|details| format!("{}-{}", details.package_name, details.version))
                    .collect();

                (runtime_deps, provided_runtime_deps)
            }
            PackageType::CrossRoot { target, .. } => (
                format_dependencies(target.prefix, package.dependencies.runtime_deps.iter())?,
                Vec::new(),
            ),
        };

        let target_prefix = match &target {
            PackageType::Host(host) => host.prefix,
            PackageType::CrossRoot { target, .. } => target.prefix,
        };

        let install_set = format_dependencies(target_prefix, package.install_set.iter())?;

        // TODO: Add this.
        let host_install_deps = Vec::new();

        let restricts = analyze_restricts(&package.details)?;
        let allow_network_access = restricts.contains(&RestrictAtom::NetworkSandbox);

        let uses = package
            .details
            .use_map
            .iter()
            .sorted_by(|(a_name, a_value), (b_name, b_value)| {
                // Enabled ones comes before disabled ones.
                a_value.cmp(b_value).reverse().then(a_name.cmp(b_name))
            })
            .map(|(name, value)| format!("{}{}", if *value { "" } else { "-" }, name))
            .join(" ");

        // The PRIMORDIAL_PACKAGES are only applicable to the board's SDK. The
        // Host SDK has all the packages already built in.
        let sdk = if PRIMORDIAL_PACKAGES
            .iter()
            .any(|package_name| package_name == &package.details.package_name)
            && matches!(target, PackageType::CrossRoot { .. })
        {
            format!("//internal/sdk/{}:base", target_prefix)
        } else {
            format!("//internal/sdk/{}", target_prefix)
        };

        let overlay = format!("//internal/overlays/{}", package.details.repo_name);

        Ok(Self {
            ebuild_name,
            basename,
            overlay,
            category: category.to_string(),
            package_name: package_name.to_string(),
            version,
            slot: package.details.slot.to_string(),
            sources,
            git_trees,
            dists,
            eclasses,
            host_build_deps,
            provided_host_build_deps,
            host_install_deps,
            target_build_deps,
            runtime_deps,
            provided_runtime_deps,
            install_set,
            allow_network_access,
            uses,
            sdk,
            direct_build_target: package.details.direct_build_target.clone(),
            has_hooks: package.details.has_hooks(),
        })
    }
}

#[derive(Serialize)]
pub struct EBuildFailure {
    ebuild_name: String,
    version: String,
    error: String,
}

impl EBuildFailure {
    pub fn new(failure: &PackageError) -> Self {
        EBuildFailure {
            ebuild_name: failure.ebuild_name.clone(),
            version: failure.version.to_string(),
            error: failure.error.to_string(),
        }
    }
}

#[derive(Serialize)]
struct BuildTemplateContext<'a> {
    target_board: Option<&'a str>,
    host_overlay_set: Option<String>,
    target_overlay_set: String,
    ebuilds: Vec<EBuildEntry>,
    failures: Vec<EBuildFailure>,
}

struct PackagesInDir<'a> {
    packages: Vec<&'a Package>,
    failed_packages: Vec<&'a PackageError>,
}

fn generate_package_build_file(
    target: &PackageType,
    packages_in_dir: &PackagesInDir,
    out: &Path,
) -> Result<()> {
    let target_board = match target {
        PackageType::Host { .. } => None,
        PackageType::CrossRoot { target, .. } => Some(target.board),
    };

    let host_overlay_set = match target {
        PackageType::Host(host) => Some(host.repo_set),
        PackageType::CrossRoot { host, .. } => host.as_ref().map(|h| h.repo_set),
    }
    .map(repository_set_to_target_path);

    let target_overlay_set = repository_set_to_target_path(match &target {
        PackageType::Host(host) => host.repo_set,
        PackageType::CrossRoot { target, .. } => target.repo_set,
    });

    let context = BuildTemplateContext {
        target_board,
        host_overlay_set,
        target_overlay_set,
        ebuilds: packages_in_dir
            .packages
            .iter()
            .map(|package| EBuildEntry::try_new(target, package))
            .collect::<Result<_>>()?,
        failures: packages_in_dir
            .failed_packages
            .iter()
            .map(|failure| EBuildFailure::new(failure))
            .collect(),
    };

    let mut file = File::create(out)?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    TEMPLATES.render_to(
        "package.BUILD.bazel",
        &tera::Context::from_serialize(context)?,
        file,
    )?;
    Ok(())
}

fn generate_package(
    target: &PackageType,
    translator: &PathTranslator,
    packages_in_dir: &PackagesInDir,
    output_dir: &Path,
) -> Result<()> {
    create_dir_all(output_dir)?;

    let ebuilds = packages_in_dir
        .packages
        .iter()
        .map(|p| &p.details.ebuild_path)
        .chain(packages_in_dir.failed_packages.iter().map(|f| &f.ebuild));

    // Create `*.ebuild` symlinks.
    for (i, ebuild) in ebuilds.enumerate() {
        let file_name = ebuild.file_name().expect("ebuild must have a file name");
        symlink(translator.to_outer(ebuild)?, output_dir.join(file_name))?;

        if i == 0 {
            // Create a `files` symlink if necessary.
            let files_dir = ebuild.with_file_name("files");
            if files_dir.try_exists()? {
                let output_files_dir = output_dir.join("files");
                symlink(translator.to_outer(files_dir)?, output_files_dir)?;
            }
        }
    }

    generate_package_build_file(target, packages_in_dir, &output_dir.join("BUILD.bazel"))?;

    Ok(())
}

/// Groups ebuilds into `<repo_name>/<category>/<package>` groups.
fn join_by_package_dir<'p>(
    all_packages: &'p [Package],
    failures: &'p [PackageError],
) -> HashMap<PathBuf, PackagesInDir<'p>> {
    let mut packages_by_dir = HashMap::<PathBuf, PackagesInDir>::new();

    let new_default = || PackagesInDir {
        packages: Vec::new(),
        failed_packages: Vec::new(),
    };

    for package in all_packages.iter() {
        packages_by_dir
            .entry(Path::new(&package.details.repo_name).join(&package.details.package_name))
            .or_insert_with(new_default)
            .packages
            .push(package);
    }

    for failure in failures.iter() {
        packages_by_dir
            .entry(Path::new(&failure.repo_name).join(&failure.package_name))
            .or_insert_with(new_default)
            .failed_packages
            .push(failure);
    }

    packages_by_dir
}

#[instrument(skip_all)]
pub fn generate_internal_packages(
    target: &PackageType,
    translator: &PathTranslator,
    all_packages: &[Package],
    failures: &[PackageError],
    output_dir: &Path,
) -> Result<()> {
    let output_packages_dir = output_dir.join("internal/packages").join(match &target {
        PackageType::Host(host) => host.prefix,
        PackageType::CrossRoot { target, .. } => target.prefix,
    });

    // Generate packages in parallel.
    let packages_by_dir = join_by_package_dir(all_packages, failures);
    packages_by_dir
        .into_par_iter()
        .try_for_each(|(relative_package_dir, packages_in_dir)| {
            let output_package_dir = output_packages_dir.join(relative_package_dir);
            generate_package(target, translator, &packages_in_dir, &output_package_dir)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: test more than just the syntax and remove this test
    #[test]
    fn template_syntax_valid() -> Result<()> {
        let context = BuildTemplateContext {
            target_board: None,
            host_overlay_set: None,
            target_overlay_set: "target_overlay_set_for_testing".to_string(),
            ebuilds: Vec::new(),
            failures: Vec::new(),
        };

        let _ = TEMPLATES.render(
            "package.BUILD.bazel",
            &tera::Context::from_serialize(context)?,
        );

        Ok(())
    }
}
