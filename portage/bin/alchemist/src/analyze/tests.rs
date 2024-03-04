// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{collections::BTreeMap, fmt::Write, path::Path, sync::Arc};

use anyhow::{Context, Result};
use tempfile::TempDir;
use version::Version;

use crate::{
    config::bundle::ConfigBundle,
    ebuild::{metadata::CachedEBuildEvaluator, CachedPackageLoader, PackageDetails, PackageLoader},
    repository::{RepositoryLayout, RepositorySet},
    resolver::PackageResolver,
};

use super::{analyze_packages, dependency::direct::DependencyExpressions, MaybePackage};

/// Provides an easy way to generate an ebuild file.
struct PackageSpec {
    category_name: String,
    short_package_name: String,
    version: Version,
    vars: BTreeMap<String, String>,
}

impl PackageSpec {
    /// Creates a new empty package spec with the specified package name and version.
    fn new(package_name: &str, version: &str) -> Result<Self> {
        let (category_name, short_package_name) = package_name
            .split_once('/')
            .with_context(|| format!("Invalid package name: {}", package_name))?;
        let version = Version::try_new(version)?;
        let default_vars = BTreeMap::from([
            ("EAPI".into(), "7".into()),
            ("KEYWORDS".into(), "*".into()),
            ("SLOT".into(), "0".into()),
        ]);
        Ok(Self {
            category_name: category_name.to_string(),
            short_package_name: short_package_name.to_string(),
            version,
            vars: default_vars,
        })
    }

    /// Defines an additional ebuild variable.
    fn var(mut self, name: &str, value: &str) -> Self {
        self.vars.insert(name.to_string(), value.to_string());
        self
    }

    /// Saves an ebuild file according to the spec.
    fn save_ebuild(&self, overlay_dir: &Path) -> Result<()> {
        let ebuild_path = overlay_dir
            .join(&self.category_name)
            .join(&self.short_package_name)
            .join(format!(
                "{}-{}.ebuild",
                &self.short_package_name, &self.version
            ));
        let ebuild_dir = ebuild_path.parent().unwrap();
        std::fs::create_dir_all(ebuild_dir)
            .with_context(|| format!("Failed to mkdir {}", ebuild_dir.display()))?;

        let mut ebuild_content = String::new();
        for (name, value) in self.vars.iter() {
            writeln!(
                &mut ebuild_content,
                "{}={}",
                name,
                shell_escape::escape(value.into())
            )?;
        }

        std::fs::write(&ebuild_path, ebuild_content)
            .with_context(|| format!("Failed to create {}", ebuild_path.display()))?;

        Ok(())
    }
}

/// Textual representation of [`PackageDependencies`] suitable for comparison.
#[derive(Clone, Debug, Eq, PartialEq)]
struct PackageDependenciesDescription {
    build_target: Vec<String>,
    test_target: Vec<String>,
    run_target: Vec<String>,
    post_target: Vec<String>,
    build_host: Vec<String>,
    install_host: Vec<String>,
    install_set: Vec<String>,
    build_host_set: Vec<String>,
}

impl PackageDependenciesDescription {
    /// The empty instance of [`PackageDependenciesDescription`].
    const EMPTY: PackageDependenciesDescription = PackageDependenciesDescription {
        build_target: Vec::new(),
        test_target: Vec::new(),
        run_target: Vec::new(),
        post_target: Vec::new(),
        build_host: Vec::new(),
        install_host: Vec::new(),
        install_set: Vec::new(),
        build_host_set: Vec::new(),
    };
}

/// Textual representation of [`MaybePackage`] suitable for comparison.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::large_enum_variant)]
enum MaybePackageDescription {
    Ok {
        /// Full package name and version number, e.g. "sys-apps/attr-2.5.1".
        package_name_version: String,

        dependencies: PackageDependenciesDescription,

        dependency_expressions: DependencyExpressions,
    },
    Err {
        /// Full package name and version number, e.g. "sys-apps/attr-2.5.1".
        package_name_version: String,

        reason: String,
    },
}

fn describe_package_list(packages: &[Arc<PackageDetails>]) -> Vec<String> {
    packages
        .iter()
        .map(|details| {
            let data = details.as_basic_data();
            format!("{}-{}", data.package_name, data.version)
        })
        .collect()
}

impl From<MaybePackage> for MaybePackageDescription {
    /// Converts [`MaybePackage`] into [`MaybePackageDescription`].
    fn from(package: MaybePackage) -> Self {
        let package_name_version = format!(
            "{}-{}",
            package.as_basic_data().package_name,
            package.as_basic_data().version
        );

        let package = match package {
            MaybePackage::Ok(p) => p,
            MaybePackage::Err(e) => {
                return Self::Err {
                    package_name_version,
                    reason: e.error.clone(),
                }
            }
        };

        let deps = &package.dependencies;
        Self::Ok {
            package_name_version,
            dependencies: PackageDependenciesDescription {
                build_target: describe_package_list(&deps.direct.build_target),
                test_target: describe_package_list(&deps.direct.test_target),
                run_target: describe_package_list(&deps.direct.run_target),
                post_target: describe_package_list(&deps.direct.post_target),
                build_host: describe_package_list(&deps.direct.build_host),
                install_host: describe_package_list(&deps.direct.install_host),
                install_set: describe_package_list(&deps.indirect.install_set),
                build_host_set: describe_package_list(&deps.indirect.build_host_set),
            },
            dependency_expressions: deps.expressions.clone(),
        }
    }
}

/// Calls [`analyze_packages`] to analyze packages for the target in unit tests.
///
/// Before calling [`analyze_packages`], it generates ebuild files with the given [`PackageSpec`].
/// After calling [`analyze_packages`], it converts the result (`Vec<MaybePackage>`) into
/// `Vec<Result<PackageDescription, String>>` for easier comparison.
fn analyze_packages_for_testing(specs: &[PackageSpec]) -> Result<Vec<MaybePackageDescription>> {
    let temp_dir = TempDir::new()?;
    let temp_dir = temp_dir.path();

    let overlay_dir = temp_dir.join("overlay");
    let tools_dir = temp_dir.join("tools");
    let src_dir = temp_dir.join("src");
    for dir in [&overlay_dir, &tools_dir, &src_dir] {
        std::fs::create_dir_all(dir)?;
    }

    // Generate ebuilds in the overlay.
    for spec in specs {
        spec.save_ebuild(&overlay_dir)?;
    }

    let repos = Arc::new(RepositorySet::load_from_layouts(
        "default",
        &[RepositoryLayout::new("chromiumos", &overlay_dir, &[])],
    )?);
    let evaluator = Arc::new(CachedEBuildEvaluator::new(
        repos.get_repos().into_iter().cloned().collect(),
        &tools_dir,
    ));

    let host_config = Arc::new(ConfigBundle::new_for_testing("host_arch"));
    let target_config = Arc::new(ConfigBundle::new_for_testing("target_arch"));

    let host_loader = Arc::new(CachedPackageLoader::new(PackageLoader::new(
        evaluator.clone(),
        host_config.clone(),
        false,
    )));
    let target_loader = Arc::new(CachedPackageLoader::new(PackageLoader::new(
        evaluator.clone(),
        target_config.clone(),
        false,
    )));

    let host_resolver = PackageResolver::new(repos.clone(), host_config.clone(), host_loader);
    let target_resolver = PackageResolver::new(repos.clone(), target_config.clone(), target_loader);

    // Analyze packages for the target.
    let packages = analyze_packages(
        &target_config,
        true,
        &src_dir,
        &host_resolver,
        &target_resolver,
    )?;

    let descriptions = packages.into_iter().map(|p| p.into()).collect();

    Ok(descriptions)
}

#[test]
fn test_analyze_packages_no_packages() -> Result<()> {
    let packages = analyze_packages_for_testing(&[])?;
    assert_eq!(packages, vec![]);
    Ok(())
}

#[test]
fn test_analyze_packages_single_no_deps() -> Result<()> {
    let packages = analyze_packages_for_testing(&[PackageSpec::new("sys-apps/hello", "1")?])?;

    assert_eq!(
        packages,
        vec![MaybePackageDescription::Ok {
            package_name_version: "sys-apps/hello-1".into(),
            dependencies: PackageDependenciesDescription {
                install_set: vec!["sys-apps/hello-1".into()],
                ..PackageDependenciesDescription::EMPTY
            },
            dependency_expressions: DependencyExpressions::default(),
        }]
    );

    Ok(())
}

#[test]
fn test_analyze_packages_normal_deps() -> Result<()> {
    //                 DEPEND                DEPEND
    // sys-apps/hello──────────►sys-libs/a────────────►sys-libs/aa
    //        │                     │        RDEPEND
    //        │                     └─────────────────►sys-libs/ab
    //        │        RDEPEND               DEPEND
    //        └────────────────►sys-libs/b────────────►sys-libs/ba
    //                              │        RDEPEND
    //                              └─────────────────►sys-libs/bb
    let packages = analyze_packages_for_testing(&[
        PackageSpec::new("sys-apps/hello", "1")?
            .var("DEPEND", "sys-libs/a")
            .var("RDEPEND", "sys-libs/b"),
        PackageSpec::new("sys-libs/a", "1")?
            .var("DEPEND", "sys-libs/aa")
            .var("RDEPEND", "sys-libs/ab"),
        PackageSpec::new("sys-libs/b", "1")?
            .var("DEPEND", "sys-libs/ba")
            .var("RDEPEND", "sys-libs/bb"),
        PackageSpec::new("sys-libs/aa", "1")?,
        PackageSpec::new("sys-libs/ab", "1")?,
        PackageSpec::new("sys-libs/ba", "1")?,
        PackageSpec::new("sys-libs/bb", "1")?,
    ])?;

    assert_eq!(
        packages,
        vec![
            MaybePackageDescription::Ok {
                package_name_version: "sys-apps/hello-1".into(),
                dependencies: PackageDependenciesDescription {
                    build_target: vec!["sys-libs/a-1".into()],
                    test_target: vec!["sys-libs/a-1".into()],
                    run_target: vec!["sys-libs/b-1".into()],
                    install_set: vec![
                        "sys-apps/hello-1".into(),
                        "sys-libs/b-1".into(),
                        "sys-libs/bb-1".into(),
                    ],
                    ..PackageDependenciesDescription::EMPTY
                },
                dependency_expressions: DependencyExpressions {
                    build_target: "sys-libs/a".into(),
                    run_target: "sys-libs/b".into(),
                    ..DependencyExpressions::default()
                }
            },
            MaybePackageDescription::Ok {
                package_name_version: "sys-libs/a-1".into(),
                dependencies: PackageDependenciesDescription {
                    build_target: vec!["sys-libs/aa-1".into()],
                    test_target: vec!["sys-libs/aa-1".into()],
                    run_target: vec!["sys-libs/ab-1".into()],
                    install_set: vec!["sys-libs/a-1".into(), "sys-libs/ab-1".into()],
                    ..PackageDependenciesDescription::EMPTY
                },
                dependency_expressions: DependencyExpressions {
                    build_target: "sys-libs/aa".into(),
                    run_target: "sys-libs/ab".into(),
                    ..DependencyExpressions::default()
                }
            },
            MaybePackageDescription::Ok {
                package_name_version: "sys-libs/aa-1".into(),
                dependencies: PackageDependenciesDescription {
                    install_set: vec!["sys-libs/aa-1".into()],
                    ..PackageDependenciesDescription::EMPTY
                },
                dependency_expressions: DependencyExpressions::default(),
            },
            MaybePackageDescription::Ok {
                package_name_version: "sys-libs/ab-1".into(),
                dependencies: PackageDependenciesDescription {
                    install_set: vec!["sys-libs/ab-1".into()],
                    ..PackageDependenciesDescription::EMPTY
                },
                dependency_expressions: DependencyExpressions::default(),
            },
            MaybePackageDescription::Ok {
                package_name_version: "sys-libs/b-1".into(),
                dependencies: PackageDependenciesDescription {
                    build_target: vec!["sys-libs/ba-1".into()],
                    test_target: vec!["sys-libs/ba-1".into()],
                    run_target: vec!["sys-libs/bb-1".into()],
                    install_set: vec!["sys-libs/b-1".into(), "sys-libs/bb-1".into()],
                    ..PackageDependenciesDescription::EMPTY
                },
                dependency_expressions: DependencyExpressions {
                    build_target: "sys-libs/ba".into(),
                    run_target: "sys-libs/bb".into(),
                    ..DependencyExpressions::default()
                }
            },
            MaybePackageDescription::Ok {
                package_name_version: "sys-libs/ba-1".into(),
                dependencies: PackageDependenciesDescription {
                    install_set: vec!["sys-libs/ba-1".into()],
                    ..PackageDependenciesDescription::EMPTY
                },
                dependency_expressions: DependencyExpressions::default(),
            },
            MaybePackageDescription::Ok {
                package_name_version: "sys-libs/bb-1".into(),
                dependencies: PackageDependenciesDescription {
                    install_set: vec!["sys-libs/bb-1".into()],
                    ..PackageDependenciesDescription::EMPTY
                },
                dependency_expressions: DependencyExpressions::default(),
            },
        ]
    );

    Ok(())
}

#[test]
fn test_analyze_packages_post_deps() -> Result<()> {
    //              PDEPEND         PDEPEND
    //            ┌──────────┐    ┌──────────┐
    //            │          ▼    │          ▼
    // sys-apps/hello      sys-libs/a      sys-libs/b
    //            ▲          │    ▲          │
    //            └──────────┘    └──────────┘
    //              RDEPEND         RDEPEND
    let packages = analyze_packages_for_testing(&[
        PackageSpec::new("sys-apps/hello", "1")?.var("PDEPEND", "sys-libs/a"),
        PackageSpec::new("sys-libs/a", "1")?
            .var("RDEPEND", "sys-apps/hello")
            .var("PDEPEND", "sys-libs/b"),
        PackageSpec::new("sys-libs/b", "1")?.var("RDEPEND", "sys-libs/a"),
    ])?;

    assert_eq!(
        packages,
        vec![
            MaybePackageDescription::Ok {
                package_name_version: "sys-apps/hello-1".into(),
                dependencies: PackageDependenciesDescription {
                    post_target: vec!["sys-libs/a-1".into()],
                    install_set: vec![
                        "sys-apps/hello-1".into(),
                        "sys-libs/a-1".into(),
                        "sys-libs/b-1".into(),
                    ],
                    ..PackageDependenciesDescription::EMPTY
                },
                dependency_expressions: DependencyExpressions {
                    post_target: "sys-libs/a".into(),
                    ..DependencyExpressions::default()
                }
            },
            MaybePackageDescription::Ok {
                package_name_version: "sys-libs/a-1".into(),
                dependencies: PackageDependenciesDescription {
                    run_target: vec!["sys-apps/hello-1".into()],
                    post_target: vec!["sys-libs/b-1".into()],
                    install_set: vec![
                        "sys-apps/hello-1".into(),
                        "sys-libs/a-1".into(),
                        "sys-libs/b-1".into(),
                    ],
                    ..PackageDependenciesDescription::EMPTY
                },
                dependency_expressions: DependencyExpressions {
                    run_target: "sys-apps/hello".into(),
                    post_target: "sys-libs/b".into(),
                    ..DependencyExpressions::default()
                }
            },
            MaybePackageDescription::Ok {
                package_name_version: "sys-libs/b-1".into(),
                dependencies: PackageDependenciesDescription {
                    run_target: vec!["sys-libs/a-1".into()],
                    install_set: vec![
                        "sys-apps/hello-1".into(),
                        "sys-libs/a-1".into(),
                        "sys-libs/b-1".into(),
                    ],
                    ..PackageDependenciesDescription::EMPTY
                },
                dependency_expressions: DependencyExpressions {
                    run_target: "sys-libs/a".into(),
                    ..DependencyExpressions::default()
                }
            },
        ]
    );

    Ok(())
}

#[test]
fn test_analyze_packages_build_host_deps() -> Result<()> {
    //                 BDEPEND
    // sys-apps/hello ────────► dev-lang/gcc
    //
    let packages = analyze_packages_for_testing(&[
        PackageSpec::new("sys-apps/hello", "1")?
            .var("IUSE", "target_arch")
            .var("REQUIRED_USE", "target_arch")
            .var("BDEPEND", "target_arch? ( dev-lang/gcc )"),
        PackageSpec::new("dev-lang/gcc", "1")?
            .var("IUSE", "host_arch")
            .var("REQUIRED_USE", "host_arch"),
    ])?;

    assert_eq!(
        packages,
        vec![
            MaybePackageDescription::Err {
                package_name_version: "dev-lang/gcc-1".into(),
                reason: "The package is masked: REQUIRED_USE not satisfied: host_arch".into(),
            },
            MaybePackageDescription::Ok {
                package_name_version: "sys-apps/hello-1".into(),
                dependencies: PackageDependenciesDescription {
                    // dev-lang/gcc is masked for target, but not for host.
                    build_host: vec!["dev-lang/gcc-1".into()],
                    install_set: vec!["sys-apps/hello-1".into()],
                    build_host_set: vec!["dev-lang/gcc-1".into()],
                    ..PackageDependenciesDescription::EMPTY
                },

                dependency_expressions: DependencyExpressions {
                    build_host: "dev-lang/gcc".into(),
                    ..DependencyExpressions::default()
                }
            },
        ]
    );

    Ok(())
}

#[test]
fn test_analyze_packages_install_host_deps() -> Result<()> {
    //                 DEPEND                   IDEPEND
    // sys-apps/hello ───────► sys-libs/libfoo ────────► sys-apps/coreutils
    //
    let packages = analyze_packages_for_testing(&[
        PackageSpec::new("sys-apps/hello", "1")?.var("DEPEND", "sys-libs/libfoo"),
        PackageSpec::new("sys-libs/libfoo", "1")?
            .var("EAPI", "8")
            .var("IDEPEND", "sys-apps/coreutils"),
        PackageSpec::new("sys-apps/coreutils", "1")?
            .var("IUSE", "host_arch")
            .var("REQUIRED_USE", "host_arch"),
    ])?;

    assert_eq!(
        packages,
        vec![
            MaybePackageDescription::Err {
                package_name_version: "sys-apps/coreutils-1".into(),
                reason: "The package is masked: REQUIRED_USE not satisfied: host_arch".into(),
            },
            MaybePackageDescription::Ok {
                package_name_version: "sys-apps/hello-1".into(),
                dependencies: PackageDependenciesDescription {
                    build_target: vec!["sys-libs/libfoo-1".into()],
                    test_target: vec!["sys-libs/libfoo-1".into()],
                    install_set: vec!["sys-apps/hello-1".into()],
                    build_host_set: vec!["sys-apps/coreutils-1".into()],
                    ..PackageDependenciesDescription::EMPTY
                },
                dependency_expressions: DependencyExpressions {
                    build_target: "sys-libs/libfoo".into(),
                    ..DependencyExpressions::default()
                }
            },
            MaybePackageDescription::Ok {
                package_name_version: "sys-libs/libfoo-1".into(),
                dependencies: PackageDependenciesDescription {
                    // sys-apps/coreutils is masked for target, but not for host.
                    install_host: vec!["sys-apps/coreutils-1".into()],
                    install_set: vec!["sys-libs/libfoo-1".into()],
                    ..PackageDependenciesDescription::EMPTY
                },
                dependency_expressions: DependencyExpressions {
                    install_host: "sys-apps/coreutils".into(),
                    ..DependencyExpressions::default()
                }
            },
        ]
    );

    Ok(())
}

#[test]
fn test_analyze_packages_install_host_deps_eapi7() -> Result<()> {
    //                  IDEPEND
    // sys-libs/libfoo ────X───► sys-apps/coreutils
    //
    let packages = analyze_packages_for_testing(&[
        PackageSpec::new("sys-libs/libfoo", "1")?
            .var("EAPI", "7")
            .var("IDEPEND", "sys-apps/coreutils"),
        PackageSpec::new("sys-apps/coreutils", "1")?,
    ])?;

    assert_eq!(
        packages,
        vec![
            MaybePackageDescription::Ok {
                package_name_version: "sys-apps/coreutils-1".into(),
                dependencies: PackageDependenciesDescription {
                    install_set: vec!["sys-apps/coreutils-1".into()],
                    ..PackageDependenciesDescription::EMPTY
                },
                dependency_expressions: DependencyExpressions::default(),
            },
            MaybePackageDescription::Ok {
                package_name_version: "sys-libs/libfoo-1".into(),
                dependencies: PackageDependenciesDescription {
                    install_set: vec!["sys-libs/libfoo-1".into()],
                    ..PackageDependenciesDescription::EMPTY
                },
                dependency_expressions: DependencyExpressions::default(),
            },
        ]
    );

    Ok(())
}

#[test]
fn test_analyze_packages_indirect_host_deps() -> Result<()> {
    //                 BDEPEND                     RDEPEND
    // sys-apps/hello ────────► sys-libs/c ────┬────────────► sys-libs/x
    //        │                                │
    //        │ DEPEND                         │
    //        ▼        IDEPEND                 │   PDEPEND
    //     sys-libs/a ────────► sys-libs/d ────┼────────────► sys-libs/y
    //        │                                │
    //        │ RDEPEND                        │
    //        ▼        IDEPEND                 │   IDEPEND
    //     sys-libs/b ────────► sys-libs/e ────┴────────────► sys-libs/z
    //        │
    //        │ PDEPEND
    //        ▼        IDEPEND
    //     sys-libs/p ────────► sys-libs/q
    let packages = analyze_packages_for_testing(&[
        PackageSpec::new("sys-apps/hello", "1")?
            .var("DEPEND", "sys-libs/a")
            .var("BDEPEND", "sys-libs/c"),
        PackageSpec::new("sys-libs/a", "1")?
            .var("EAPI", "8")
            .var("RDEPEND", "sys-libs/b")
            .var("IDEPEND", "sys-libs/d"),
        PackageSpec::new("sys-libs/b", "1")?
            .var("EAPI", "8")
            .var("IDEPEND", "sys-libs/e")
            .var("PDEPEND", "sys-libs/p"),
        PackageSpec::new("sys-libs/c", "1")?
            .var("EAPI", "8")
            .var("RDEPEND", "sys-libs/x")
            .var("PDEPEND", "sys-libs/y")
            .var("IDEPEND", "sys-libs/z"),
        PackageSpec::new("sys-libs/d", "1")?
            .var("EAPI", "8")
            .var("RDEPEND", "sys-libs/x")
            .var("PDEPEND", "sys-libs/y")
            .var("IDEPEND", "sys-libs/z"),
        PackageSpec::new("sys-libs/e", "1")?
            .var("EAPI", "8")
            .var("RDEPEND", "sys-libs/x")
            .var("PDEPEND", "sys-libs/y")
            .var("IDEPEND", "sys-libs/z"),
        PackageSpec::new("sys-libs/p", "1")?
            .var("EAPI", "8")
            .var("IDEPEND", "sys-libs/q"),
        PackageSpec::new("sys-libs/q", "1")?,
        PackageSpec::new("sys-libs/x", "1")?,
        PackageSpec::new("sys-libs/y", "1")?,
        PackageSpec::new("sys-libs/z", "1")?,
    ])?;

    let hello_package = packages
        .into_iter()
        .find(|p| {
            matches!(
                p,
                MaybePackageDescription::Ok { package_name_version, .. }
                if package_name_version == "sys-apps/hello-1"
            )
        })
        .unwrap();

    assert_eq!(
        hello_package,
        MaybePackageDescription::Ok {
            package_name_version: "sys-apps/hello-1".into(),
            dependencies: PackageDependenciesDescription {
                build_target: vec!["sys-libs/a-1".into()],
                test_target: vec!["sys-libs/a-1".into()],
                build_host: vec!["sys-libs/c-1".into()],
                install_set: vec!["sys-apps/hello-1".into()],
                build_host_set: vec![
                    "sys-libs/c-1".into(),
                    "sys-libs/d-1".into(),
                    "sys-libs/e-1".into()
                ],
                ..PackageDependenciesDescription::EMPTY
            },
            dependency_expressions: DependencyExpressions {
                build_target: "sys-libs/a".into(),
                build_host: "sys-libs/c".into(),
                ..DependencyExpressions::default()
            }
        },
    );

    Ok(())
}

#[test]
fn test_analyze_packages_propagate_errors() -> Result<()> {
    //                 DEPEND              RDEPEND
    // sys-apps/hello ───────► sys-libs/a ────────► sys-libs/b
    //
    let packages = analyze_packages_for_testing(&[
        PackageSpec::new("sys-apps/hello", "1")?.var("DEPEND", "sys-libs/a"),
        PackageSpec::new("sys-libs/a", "1")?.var("RDEPEND", "sys-libs/b"),
        PackageSpec::new("sys-libs/b", "1")?
            .var("IUSE", "host_arch")
            .var("REQUIRED_USE", "host_arch"),
    ])?;

    assert_eq!(
        packages,
        vec![
            MaybePackageDescription::Err {
                package_name_version: "sys-apps/hello-1".into(),
                reason: "Failed to analyze sys-libs/a-1: Resolving runtime dependencies \
                for sys-libs/a-1: Unsatisfiable dependency: No package satisfies sys-libs/b"
                    .into(),
            },
            MaybePackageDescription::Err {
                package_name_version: "sys-libs/a-1".into(),
                reason: "Resolving runtime dependencies for sys-libs/a-1: \
                Unsatisfiable dependency: No package satisfies sys-libs/b"
                    .into(),
            },
            MaybePackageDescription::Err {
                package_name_version: "sys-libs/b-1".into(),
                reason: "The package is masked: REQUIRED_USE not satisfied: host_arch".into(),
            },
        ]
    );

    Ok(())
}

#[test]
fn test_analyze_dep_xpak_values() -> Result<()> {
    let depend = "
        !!app-crypt/heimdal
        !cros_host? ( chromeos-base/crosid:= )
        cros_host? ( chromeos-base/cros-config-host:= )
        app-arch/libarchive:2=
        dev-libs/libzip:=
        >=sys-fs/e2fsprogs-1.46.4-r51[target_arch(-)]
        || (
            >=dev-libs/libverto-0.2.5[libev,target_arch(-)]
            >=dev-libs/libverto-0.2.5[libevent,target_arch(-)]
        )
    ";
    let packages = analyze_packages_for_testing(&[
        PackageSpec::new("sys-libs/libfoo", "1")?
            .var("EAPI", "7")
            .var("DEPEND", depend)
            .var("RDEPEND", &format!("{depend} sys-apps/coreboot-utils:=")),
        PackageSpec::new("app-crypt/heimdal", "1")?,
        PackageSpec::new("chromeos-base/crosid", "2")?.var("SLOT", "0/a"),
        PackageSpec::new("chromeos-base/cros-config-host", "3")?.var("SLOT", "1/b"),
        PackageSpec::new("app-arch/libarchive", "4")?.var("SLOT", "2/c"),
        PackageSpec::new("dev-libs/libzip", "5")?.var("SLOT", "3/d"),
        PackageSpec::new("sys-fs/e2fsprogs", "1.50")?
            .var("SLOT", "4/e")
            .var("IUSE", "target_arch"),
        PackageSpec::new("dev-libs/libverto", "0.3")?
            .var("SLOT", "5/f")
            .var("IUSE", "target_arch libev +libevent"),
        PackageSpec::new("sys-apps/coreboot-utils", "6")?.var("SLOT", "6/g"),
    ])?;

    assert_eq!(
        packages.last().unwrap(),
        &MaybePackageDescription::Ok {
            package_name_version: "sys-libs/libfoo-1".into(),
            dependencies: PackageDependenciesDescription {
                build_target: vec![
                    "app-arch/libarchive-4".into(),
                    "chromeos-base/crosid-2".into(),
                    "dev-libs/libverto-0.3".into(),
                    "dev-libs/libzip-5".into(),
                    "sys-fs/e2fsprogs-1.50".into()
                ],
                test_target: vec![
                    "app-arch/libarchive-4".into(),
                    "chromeos-base/crosid-2".into(),
                    "dev-libs/libverto-0.3".into(),
                    "dev-libs/libzip-5".into(),
                    "sys-fs/e2fsprogs-1.50".into()
                ],
                run_target: vec![
                    "app-arch/libarchive-4".into(),
                    "chromeos-base/crosid-2".into(),
                    "dev-libs/libverto-0.3".into(),
                    "dev-libs/libzip-5".into(),
                    "sys-apps/coreboot-utils-6".into(),
                    "sys-fs/e2fsprogs-1.50".into()
                ],
                install_set: vec![
                    "app-arch/libarchive-4".into(),
                    "chromeos-base/crosid-2".into(),
                    "dev-libs/libverto-0.3".into(),
                    "dev-libs/libzip-5".into(),
                    "sys-apps/coreboot-utils-6".into(),
                    "sys-fs/e2fsprogs-1.50".into(),
                    "sys-libs/libfoo-1".into()
                ],
                ..PackageDependenciesDescription::EMPTY
            },
            //
            dependency_expressions: DependencyExpressions {
                // The any-of constraints should probably be resolved for
                // build dependencies, but we are keeping portage's behavior
                // for now.
                build_target: concat!(
                    "!!app-crypt/heimdal ",
                    "chromeos-base/crosid:0/a= ",
                    "app-arch/libarchive:2/c= ",
                    "dev-libs/libzip:3/d= ",
                    ">=sys-fs/e2fsprogs-1.46.4-r51[target_arch(-)] ",
                    "|| ( >=dev-libs/libverto-0.2.5[libev,target_arch(-)] >=dev-libs/libverto-0.2.5[libevent,target_arch(-)] )"
                ).into(),
                run_target: concat!(
                    "!!app-crypt/heimdal ",
                    "chromeos-base/crosid:0/a= ",
                    "app-arch/libarchive:2/c= ",
                    "dev-libs/libzip:3/d= ",
                    ">=sys-fs/e2fsprogs-1.46.4-r51[target_arch(-)] ",
                    // Any-of in runtime deps only makes sense if the
                    // package didn't link against a specific dep. In theory
                    // we should have a := in here, but the PMS disallows
                    // the := operator in any-of constraints.
                    "|| ( >=dev-libs/libverto-0.2.5[libev,target_arch(-)] >=dev-libs/libverto-0.2.5[libevent,target_arch(-)] ) ",
                    "sys-apps/coreboot-utils:6/g="
                ).into(),
                ..DependencyExpressions::default()
            }
        }
    );

    Ok(())
}
