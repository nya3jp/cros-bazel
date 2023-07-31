// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::sync::Arc;

use anyhow::{Context, Result};
use itertools::Itertools;

use crate::{
    bash::vars::BashValue,
    data::UseMap,
    dependency::{
        algorithm::{elide_use_conditions, parse_simplified_dependency, simplify},
        package::{PackageBlock, PackageDependency},
        CompositeDependency, Dependency,
    },
    ebuild::PackageDetails,
    resolver::PackageResolver,
};

/// Analyzed package dependencies of a package. It is returned by
/// [`analyze_dependencies`].
///
/// This struct represents dependencies as lists of [`PackageDetails`] instead
/// of [`PackageDependency`] that can contain complex expressions such as
/// any-of.
#[derive(Clone, Debug)]
pub struct PackageDependencies {
    pub build_deps: Vec<Arc<PackageDetails>>,
    pub runtime_deps: Vec<Arc<PackageDetails>>,
    pub post_deps: Vec<Arc<PackageDetails>>,
    pub build_host_deps: Vec<Arc<PackageDetails>>,
    pub install_host_deps: Vec<Arc<PackageDetails>>,
}

/// Represents a package dependency type.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum DependencyKind {
    /// Build-time dependencies, aka "DEPEND" in Portage.
    Build,
    /// Run-time dependencies, aka "RDEPEND" in Portage.
    Run,
    /// Post-time dependencies, aka "PDEPEND" in Portage.
    Post,
    /// Build-time host tool dependencies, aka "BDEPEND" in Portage.
    /// `cross_compile` will be true when CBUILD != CHOST.
    BuildHost { cross_compile: bool },
    /// Install-time host tool dependencies, aka "IDEPEND" in Portage.
    /// `cross_compile` will be true when CBUILD != CHOST.
    InstallHost { cross_compile: bool },
}

/// Parses a dependency represented as [`PackageDependency`] that can contain
/// complex expressions such as any-of to a simple list of [`PackageDetails`].
fn parse_dependencies(
    deps: PackageDependency,
    use_map: &UseMap,
    resolver: &PackageResolver,
    allow_list: Option<&[&str]>,
) -> Result<Vec<Arc<PackageDetails>>> {
    let deps = elide_use_conditions(deps, use_map).unwrap_or_default();

    // Rewrite atoms.
    let deps = deps.try_map_tree_par(|dep| -> Result<PackageDependency> {
        match dep {
            Dependency::Leaf(atom) => {
                // Remove blocks.
                if atom.block() != PackageBlock::None {
                    return Ok(Dependency::new_constant(
                        true,
                        &format!("Package block {} is ignored", atom),
                    ));
                }

                // Remove packages not specified in the allow list.
                // This is a work around for EAPI < 7 packages that don't
                // support BDEPENDs.
                if let Some(allow_list) = allow_list {
                    if !allow_list.contains(&atom.package_name()) {
                        return Ok(Dependency::new_constant(
                            true,
                            &format!("Package {} is not in allowed list", atom),
                        ));
                    }
                }

                // Remove provided packages.
                if resolver.find_provided_packages(&atom).next().is_some() {
                    return Ok(Dependency::new_constant(
                        true,
                        &format!("Package {} is in package.provided", atom),
                    ));
                }

                // Remove non-existent packages.
                match resolver.find_best_package_dependency(use_map, &atom) {
                    Ok(result) => {
                        if result.is_none() {
                            return Ok(Dependency::new_constant(
                                false,
                                &format!("No package satisfies {}", atom),
                            ));
                        };
                    }
                    Err(err) => {
                        return Ok(Dependency::new_constant(
                            false,
                            &format!("Error matching {}: {:?}", atom, err),
                        ));
                    }
                }

                Ok(Dependency::Leaf(atom))
            }
            _ => Ok(dep),
        }
    })?;

    let deps = simplify(deps);

    // Resolve any-of dependencies by picking the first item.
    let deps = deps.map_tree_par(|dep| match dep {
        Dependency::Composite(composite) => match *composite {
            CompositeDependency::AnyOf { children } if !children.is_empty() => {
                children.into_iter().next().unwrap()
            }
            other => Dependency::new_composite(other),
        },
        other => other,
    });

    let deps = simplify(deps);

    let atoms = parse_simplified_dependency(deps)?;

    atoms
        .into_iter()
        .map(|atom| {
            Ok(
                resolver
                    .find_best_package_dependency(use_map, &atom)?
                    .expect("package to exist"), // missing packages were filtered above
            )
        })
        .collect::<Result<Vec<_>>>()
}

// TODO: Remove this hack.
fn get_extra_dependencies(details: &PackageDetails, kind: DependencyKind) -> &'static str {
    match (details.package_name.as_str(), kind) {
        // poppler seems to support building without Boost, but the build fails
        // without it.
        ("app-text/poppler", DependencyKind::Build) => "dev-libs/boost",
        // m2crypt fails to build for missing Python.h.
        ("dev-python/m2crypto", DependencyKind::Build) => "dev-lang/python:3.6",
        // xau.pc contains "Requires: xproto", so it should be listed as RDEPEND.
        ("x11-libs/libXau", DependencyKind::Run) => "x11-base/xorg-proto",

        // x11-misc/compose-tables requires the unprefixed cpp located at
        // /usr/bin/cpp. This symlink points to `clang-cpp`, but the symlink
        // is created by the `gcc` package. x11-misc/compose-tables doesn't
        // actually use GCC for anything other than this symlink.
        // See b/258234653
        ("x11-misc/compose-tables", DependencyKind::BuildHost { .. }) => "sys-devel/gcc",
        ("x11-libs/libX11", DependencyKind::BuildHost { .. }) => "sys-devel/gcc",

        // The nls use flag claims that gettext is optional, but in reality
        // the ./configure script calls `aclocal` and it expects the gettext
        // macros.
        ("media-libs/libexif", DependencyKind::BuildHost { .. }) => "sys-devel/gettext",

        /*
         * /build/arm64-generic/tmp/portage/sys-fs/fuse-2.9.8-r5/work/fuse-2.9.8/missing: line 81: aclocal-1.15: command not found
         * CDPATH="${ZSH_VERSION+.}:" && cd . && /bin/sh /build/arm64-generic/tmp/portage/sys-fs/fuse-2.9.8-r5/work/fuse-2.9.8/missing aclocal-1.15 -I m4
         * configure.ac:74: warning: macro 'AM_ICONV' not found in library
         */
        ("sys-fs/fuse", DependencyKind::BuildHost { .. }) => "sys-devel/automake sys-devel/gettext",

        /*
         * checking host system type... Invalid configuration `aarch64-cros-linux-gnu': machine `aarch64-cros' not recognized
         */
        ("dev-libs/libdaemon", DependencyKind::BuildHost { .. }) => "sys-devel/gnuconfig",

        /*
         * aclocal-1.15: command not found
         *  ./Configure: line 39: which: command not found
         */
        ("sys-processes/lsof", DependencyKind::BuildHost { .. }) => {
            "sys-devel/automake sys-apps/which"
        }

        /*
         *  configure.ac:36: warning: macro 'AM_ICONV' not found in library
         */
        ("app-arch/cabextract", DependencyKind::BuildHost { .. }) => "sys-devel/gettext",

        // When cross compiling `dev-libs/nss`, it requires `dev-libs/nss` to be
        // installed on the build host. We can't add `dev-libs/nss` as a BDEPEND
        // to the ebuild because that would cause a circular dependency when
        // building for the host.
        // See: https://bugs.gentoo.org/759127
        (
            "dev-libs/nss",
            DependencyKind::BuildHost {
                cross_compile: true,
            },
        ) => "dev-libs/nss",
        // dev-libs/nss needs to run the `shlibsign` binary when installing.
        // When cross-compiling that means we need need to use the build host's
        // `shlibsign`.
        (
            "dev-libs/nss",
            DependencyKind::InstallHost {
                cross_compile: true,
            },
        ) => "dev-libs/nss",

        /*
         * make[2]: Entering directory '/build/arm64-generic/tmp/portage/net-libs/rpcsvc-proto-1.3.1-r4/work/rpcsvc-proto-1.3.1/rpcsvc'
         * rpcgen -h -o klm_prot.h klm_prot.x
         * make[2]: rpcgen: Command not found
         */
        (
            "net-libs/rpcsvc-proto",
            DependencyKind::BuildHost {
                cross_compile: true,
            },
        ) => "net-libs/rpcsvc-proto",

        /*
         * cannot find C preprocessor: cpp
         *
         * We use gcc for `cpp`, we should switch this to clang.
         * /usr/bin/x86_64-pc-linux-gnu-cpp: symbolic link to /usr/x86_64-pc-linux-gnu/gcc-bin/10.2.0/x86_64-pc-linux-gnu-cpp
         */
        (
            "net-libs/rpcsvc-proto",
            DependencyKind::BuildHost {
                cross_compile: false,
            },
        ) => "sys-devel/gcc",

        /*
         * configure: WARNING: nih-dbus-tool not found, but you are cross-compiling.  Using built copy, which is probably not what you want.  Set NIH_DBUS_TOOL maybe?
         */
        (
            "sys-libs/libnih",
            DependencyKind::BuildHost {
                cross_compile: true,
            },
        ) => "sys-libs/libnih",

        /*
         * bc -c ./libmath.b </dev/null >libmath.h
         * /bin/sh: line 1: bc: command not found
         */
        (
            "sys-devel/bc",
            DependencyKind::BuildHost {
                cross_compile: true,
            },
        ) => "sys-devel/bc",

        /*
         * /bin/sh: line 2: -F/build/arm64-generic/tmp/portage/sys-apps/groff-1.22.4-r2/work/groff-1.22.4/font: No such file or directory
         */
        (
            "sys-apps/groff",
            DependencyKind::BuildHost {
                cross_compile: true,
            },
        ) => "sys-apps/groff",

        /*
         * /bin/sh: line 1: bc: command not found
         * make[2]: *** [/mnt/host/source/src/third_party/kernel/v5.15/./Kbuild:24: include/generated/timeconst.h] Error 127
         *
         * /bin/sh: line 1: perl: command not found
         * make[2]: *** [/mnt/host/source/src/third_party/kernel/v5.15/lib/Makefile:323: lib/oid_registry_data.c] Error 127
         *
         * /build/arm64-generic/tmp/portage/sys-kernel/chromeos-kernel-5_15-9999/temp/environment: line 1659: lz4: command not found
         *
         * /build/arm64-generic/tmp/portage/sys-kernel/chromeos-kernel-5_15-9999/temp/environment: line 1748: fdtget: command not found
         *
         * /build/arm64-generic/tmp/portage/sys-kernel/chromeos-kernel-5_15-9999/temp/environment: line 2436: mkimage: command not found
         *
         * TODO: Update cros-kernel eclass
         */
        ("sys-kernel/chromeos-kernel-5_15", DependencyKind::BuildHost { .. }) => {
            "sys-devel/bc dev-lang/perl app-arch/lz4 sys-apps/dtc dev-embedded/u-boot-tools"
        }

        /*
         * checking for compile_et... no
         * configure: error: cannot find compile_et
         * ### /build/arm64-generic/tmp/portage/app-crypt/mit-krb5-1.20.1/work/krb5-1.20.1/src-.arm64/config.log:
         */
        ("app-crypt/mit-krb5", DependencyKind::BuildHost { .. }) => "sys-fs/e2fsprogs",

        /*
         * configure:13038: error: possibly undefined macro: AC_LIB_PREPARE_PREFIX
         */
        ("media-libs/libmtp", DependencyKind::BuildHost { .. }) => "sys-devel/gettext",

        /*
         * /bin/sh: line 1: glib-mkenums: command not found
         * make: *** [Makefile:1301: gudev/gudevenumtypes.c] Error 127
         */
        ("dev-libs/libgudev", DependencyKind::BuildHost { .. }) => "dev-util/glib-utils",

        /*
         *  *    brltty_config ...
         * /usr/bin/env: ‘tclsh’: No such file or directory
         */
        ("app-accessibility/brltty", DependencyKind::BuildHost { .. }) => "dev-lang/tcl",

        /*
         * perl ./xml2lst.pl < evdev.xml > evdev.lst
         * /bin/sh: line 1: perl: command not found
         */
        ("x11-misc/xkeyboard-config", DependencyKind::BuildHost { .. }) => "dev-lang/perl",

        /*
         * ./Configure: line 39: which: command not found
         * ./Configure: line 2873: perl: command not found
         */
        ("sys-process/lsof", DependencyKind::BuildHost { .. }) => "dev-lang/perl sys-apps/which",

        /*
         * /build/arm64-generic/tmp/portage/sys-fs/ecryptfs-utils-108-r5/temp/environment: line 876: intltoolize: command not found
         */
        ("sys-fs/ecryptfs-utils", DependencyKind::BuildHost { .. }) => "dev-util/intltool",

        /*
         * /bin/sh: line 15: soelim: command not found
         */
        ("net-nds/openldap", DependencyKind::BuildHost { .. }) => "sys-apps/groff",

        /*
         * Our fake sudo package uses dataclasses which is a python 3.8 feature.
         * We need to add the dep when a target package uses sudo :/
         *
         * We need gcc because chrome uses a bundled ninja that is built against libstdc++.
         *
         * /home/root/chrome_root/src/third_party/ninja/ninja: error while loading shared libraries: libstdc++.so.6: cannot open shared object file: No such file or directory
         */
        ("chromeos-base/chrome-icu", DependencyKind::BuildHost { .. }) => {
            "dev-python/dataclasses sys-devel/gcc"
        }
        ("chromeos-base/chromeos-chrome", DependencyKind::BuildHost { .. }) => {
            "dev-python/dataclasses sys-devel/gcc"
        }

        /*
         * /build/arm64-generic/tmp/portage/net-libs/libmbim-9999/temp/environment: line 3552: git: command not found
         *
         * So this one is annoying. It's an EAPI 6 ebuild, so it doesn't get the git BDEPEND,
         * but we really only need git to get the VCS_ID. We need to update the cros-workon
         * eclass to stop calling git if there is no .git directory.
         */
        ("net-libs/libmbim", DependencyKind::BuildHost { .. }) => "dev-vcs/git",
        ("media-libs/minigbm", DependencyKind::BuildHost { .. }) => "dev-vcs/git",
        ("media-libs/cros-camera-hal-usb", DependencyKind::BuildHost { .. }) => "dev-vcs/git",

        /* Our setuptools is way too old. b/293899573 */
        ("dev-python/jinja", DependencyKind::BuildHost { .. }) => "dev-python/markupsafe",

        /*
         * /var/tmp/portage/sys-libs/binutils-libs-2.37_p1-r1/work/binutils-2.37/missing: line 81: makeinfo: command not found
         */
        ("sys-libs/binutils-libs", DependencyKind::BuildHost { .. }) => "sys-apps/texinfo",

        /*
         * make[1]: flex: Command not found
         */
        ("sys-libs/libsepol", DependencyKind::BuildHost { .. }) => "sys-devel/flex",

        /* TODO: I lost the error message */
        ("sys-fs/lvm2", DependencyKind::BuildHost { .. }) => "sys-apps/which sys-devel/binutils",
        ("x11-misc/compose-tables", DependencyKind::Build) => "x11-misc/util-macros",

        _ => "",
    }
}

fn extract_dependencies(
    details: &PackageDetails,
    kind: DependencyKind,
    resolver: &PackageResolver,
    allow_list: Option<&[&str]>,
) -> Result<Vec<Arc<PackageDetails>>> {
    let var_name = match kind {
        DependencyKind::Build => "DEPEND",
        DependencyKind::Run => "RDEPEND",
        DependencyKind::Post => "PDEPEND",
        DependencyKind::BuildHost { .. } => "BDEPEND",
        DependencyKind::InstallHost { .. } => "IDEPEND",
    };

    let raw_deps = details.vars.get_scalar_or_default(var_name)?;

    let raw_extra_deps = get_extra_dependencies(details, kind);

    let joined_raw_deps = format!("{} {}", raw_deps, raw_extra_deps);
    let deps = joined_raw_deps.parse::<PackageDependency>()?;

    parse_dependencies(deps, &details.use_map, resolver, allow_list)
}

// TODO: Remove this hack.
fn is_rust_source_package(details: &PackageDetails) -> bool {
    let is_rust_package = details.inherited.contains("cros-rust");
    let is_cros_workon_package = details.inherited.contains("cros-workon");
    let has_src_compile = matches!(
        details.vars.hash_map().get("HAS_SRC_COMPILE"),
        Some(BashValue::Scalar(s)) if s == "1");

    is_rust_package && !is_cros_workon_package && !has_src_compile
}

// We don't want to open the flood gates and pull in ALL DEPENDs
// because there are only a handful that are actually BDEPENDs.
// We keep a hand curated list of packages that are known to be
// BDEPENDs. Ideally we upgrade all ebuilds to EAPI7 and delete this
// block, but that's a lot of work.
static DEPEND_AS_BDEPEND_ALLOW_LIST: [&str; 21] = [
    "app-misc/jq",
    "app-portage/elt-patches",
    "dev-lang/perl",
    "dev-perl/XML-Parser",
    "dev-python/m2crypto",
    "dev-util/cmake",
    "dev-util/meson",
    "dev-util/meson-format-array",
    "dev-util/ninja",
    "dev-vcs/git", // TODO: We need to make cros-workon stop calling `git`.
    "sys-apps/texinfo",
    "sys-devel/autoconf",
    "sys-devel/autoconf-archive",
    "sys-devel/automake",
    "sys-devel/bison",
    "sys-devel/flex",
    "sys-devel/gnuconfig",
    "sys-devel/libtool",
    "sys-devel/m4",
    "sys-devel/make",
    "virtual/yacc",
];

/// Analyzes ebuild variables and returns [`PackageDependencies`] containing
/// its dependencies as a list of [`PackageDetails`].
pub fn analyze_dependencies(
    details: &PackageDetails,
    cross_compile: bool,
    host_resolver: Option<&PackageResolver>,
    target_resolver: &PackageResolver,
) -> Result<PackageDependencies> {
    let build_deps = extract_dependencies(details, DependencyKind::Build, target_resolver, None)
        .with_context(|| {
            format!(
                "Resolving build-time dependencies for {}-{}",
                &details.package_name, &details.version
            )
        })?;

    let runtime_deps = extract_dependencies(details, DependencyKind::Run, target_resolver, None)
        .with_context(|| {
            format!(
                "Resolving runtime dependencies for {}-{}",
                &details.package_name, &details.version
            )
        })?;

    let build_host_deps = if let Some(host_resolver) = host_resolver {
        // We query BDEPEND regardless of EAPI because we want our overrides
        // from `get_extra_dependencies` to allow specifying a BDEPEND even
        // if the EAPI doesn't support it.
        let mut build_host_deps = extract_dependencies(
            details,
            DependencyKind::BuildHost { cross_compile },
            host_resolver,
            None,
        )
        .with_context(|| {
            format!(
                "Resolving build-time host dependencies for {}-{}",
                &details.package_name, &details.version
            )
        })?;

        if !details.supports_bdepend() {
            // We need to apply the allow list filtering during dependency
            // evaluation instead of post-dependency evaluation because
            // there are dependencies that we can't satisfy using the host
            // resolver. i.e. `libchrome[cros_debug=]`.
            let build_deps_for_host = extract_dependencies(
                details,
                DependencyKind::Build,
                host_resolver,
                Some(&DEPEND_AS_BDEPEND_ALLOW_LIST),
            )
            .with_context(|| {
                format!(
                    "Resolving build-time dependencies as host dependencies for {}-{}",
                    &details.package_name, &details.version
                )
            })?;

            for package_details in build_deps_for_host {
                if !build_host_deps
                    .iter()
                    .any(|a| a.ebuild_path == package_details.ebuild_path)
                {
                    build_host_deps.push(package_details);
                }
            }
        }

        build_host_deps
    } else {
        vec![]
    };

    let install_host_deps = if let Some(host_resolver) = host_resolver {
        extract_dependencies(
            details,
            DependencyKind::InstallHost { cross_compile },
            host_resolver,
            None,
        )
        .with_context(|| {
            format!(
                "Resolving install-time host dependencies for {}-{}",
                &details.package_name, &details.version
            )
        })?
    } else {
        vec![]
    };

    // Some Rust source packages have their dependencies only listed as DEPEND.
    // They also need to be listed as RDPEND so they get pulled in as transitive
    // deps.
    // TODO: Fix ebuilds and remove this hack.
    let runtime_deps = if is_rust_source_package(details) {
        runtime_deps
            .into_iter()
            .chain(build_deps.clone().into_iter())
            .sorted_by(|a, b| {
                a.package_name
                    .cmp(&b.package_name)
                    .then(a.version.cmp(&b.version))
            })
            .dedup_by(|a, b| a.package_name == b.package_name && a.version == b.version)
            .collect()
    } else {
        runtime_deps
    };

    let post_deps = extract_dependencies(details, DependencyKind::Post, target_resolver, None)
        .with_context(|| {
            format!(
                "Resolving post-time dependencies for {}-{}",
                &details.package_name, &details.version
            )
        })?;

    Ok(PackageDependencies {
        build_deps,
        runtime_deps,
        post_deps,
        build_host_deps,
        install_host_deps,
    })
}
