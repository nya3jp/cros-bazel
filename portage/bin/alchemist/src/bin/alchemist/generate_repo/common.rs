// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{path::Path, sync::Arc};

use alchemist::{
    analyze::{
        dependency::PackageDependencies,
        source::{PackageDistSource, PackageSources},
    },
    ebuild::{PackageDetails, PackageMetadataError},
    repository::RepositorySet,
};
use anyhow::Result;
use itertools::Itertools;
use serde::Serialize;
use version::Version;

pub static AUTOGENERATE_NOTICE: &str = "# AUTO-GENERATED FILE. DO NOT EDIT.\n\n";

/// Packages that are used to bootstrap the target's SYSROOT.
///
/// These packages are considered implicit built-time dependencies for all
/// packages built for the target.
///
/// TODO: Create a virtual package containing this list instead of hard coding
/// it into alchemist.
pub static PRIMORDIAL_PACKAGES: &[&str] = &[
    "sys-kernel/linux-headers",
    "sys-libs/gcc-libs",
    // We currently have glibc in package.provided for !amd64-host boards.
    // When cross-compiling we use the toolchain's glibc package and manually
    // install it into the SYSROOT.
    "sys-libs/glibc",
    "sys-libs/libcxx",
    "sys-libs/llvm-libunwind",
    "virtual/os-headers",
];

/// The packages we install to create a cross-compiler toolchain layer.
///
/// When cross-compiling a sysroot we need to ensure the cross-compiler
/// toolchain is implicitly provided. Portage will use the target's CHOST
/// variable to derive the compiler paths.
///
/// e.g.,
///     CHOST=aarch64-cros-linux-gnu
///     CC="${CHOST}-clang"
///
/// This unfortunately means that it's not possible for individual packages to
/// declare an explicit dependency on the specific cross-compiler tools they
/// depend on.
///
/// i.e., The following is not possible because the CHOST variable is profile
/// dependent.
///     BDEPEND="cross-${CHOST}/go"
///
/// This means we need to include `go` and `gcc` as implicit dependencies for
/// ALL target packages even though only a subset of packages actually require
/// these host tools.
///
/// Questions:
/// * Why is LLVM not listed here? The sys-devel/llvm ebuild generates compilers
///   for all the different architectures we support.
/// * Why is the compiler-rt library not listed as a PRIMORDIAL_PACKAGES list
///   instead? The compiler-rt is a host package that provides helper objects
///   for the `llvm` cross-compilers. We don't want it installed into the
///   target's sysroot.
///
/// Packages that don't have a category specified will default to
/// `cross-$CHOST`.
pub static TOOLCHAIN_PACKAGE_NAMES: &[&str] = &[
    "binutils",
    // Only used by the packages that call `cros_use_gcc`.
    "gcc",
    "go",
    // compiler-rt is only required for non-x86 toolchains. It's handled
    // as a special case in the generator code.
    "compiler-rt",
    // The crossdev package provides /usr/share/config.site which includes
    // a bunch of autoconf overrides that are used when cross compiling.
    "sys-devel/crossdev",
];

fn file_name_to_name(file_name: &str) -> String {
    let escaped_file_name: String = file_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '.' {
                c.to_string()
            } else {
                format!("_{:x}_", c as u32)
            }
        })
        .join("");
    format!("dist_{}", escaped_file_name)
}

pub fn repository_set_to_target_path(repo_set: &RepositorySet) -> String {
    format!("//internal/overlays:{}", repo_set.primary().name())
}

pub fn package_details_to_target_path(details: &PackageDetails, prefix: &str) -> String {
    format!(
        "//internal/packages/{}/{}/{}:{}",
        prefix, details.repo_name, details.package_name, details.version
    )
}

pub fn package_details_to_package_set_target_path(
    details: &PackageDetails,
    prefix: &str,
) -> String {
    format!(
        "{}_package_set",
        package_details_to_target_path(details, prefix)
    )
}

/// Holds rich information about a package.
pub struct Package {
    /// Package information extracted by [`PackageResolver`].
    pub details: Arc<PackageDetails>,

    /// Dependency information computed from the package metadata.
    pub dependencies: PackageDependencies,

    /// Locates source code needed to build this package.
    pub sources: PackageSources,

    /// A list of packages needed to install together with this package.
    /// Specifically, it is a transitive closure of dependencies introduced by
    /// RDEPEND and PDEPEND. Alchemist needs to compute it, instead of letting
    /// Bazel compute it, because there can be circular dependencies.
    pub install_set: Vec<Arc<PackageDetails>>,

    /// The BDEPENDs declared by this package and all the IDEPENDs specified
    /// by the package's DEPENDs and their transitive RDEPENDs.
    ///
    /// When building the `build_deps` SDK layer, we need to ensure that all
    /// the IDEPENDs are installed into the `build_host_deps` SDK layer. We
    /// Could add the concept of an IDEPEND to bazel, but it would make the
    /// `sdk_install_deps` rule very complicated and harder to understand.
    pub build_host_deps: Vec<Arc<PackageDetails>>,
}

/// Holds information for packages whose metadata was loaded successfully, but
/// the analysis failed.
#[derive(Clone, Debug)]
pub struct PackageAnalysisError {
    pub details: Arc<PackageDetails>,
    pub error: String,
}

#[derive(Clone, Debug)]
pub enum PackageError {
    PackageMetadataError(PackageMetadataError),
    PackageAnalysisError(PackageAnalysisError),
}

impl<'a> PackageError {
    pub fn repo_name(&self) -> &str {
        match self {
            Self::PackageMetadataError(p) => &p.repo_name,
            Self::PackageAnalysisError(p) => &p.details.repo_name,
        }
    }
    pub fn package_name(&self) -> &str {
        match self {
            Self::PackageMetadataError(p) => &p.package_name,
            Self::PackageAnalysisError(p) => &p.details.package_name,
        }
    }
    pub fn ebuild(&self) -> &Path {
        match self {
            Self::PackageMetadataError(p) => &p.ebuild,
            Self::PackageAnalysisError(p) => &p.details.ebuild_path,
        }
    }
    pub fn version(&self) -> &Version {
        match self {
            Self::PackageMetadataError(p) => &p.version,
            Self::PackageAnalysisError(p) => &p.details.version,
        }
    }
    pub fn error(&self) -> &str {
        match self {
            Self::PackageMetadataError(p) => &p.error,
            Self::PackageAnalysisError(p) => &p.error,
        }
    }
}

#[derive(Serialize)]
pub struct DistFileEntry {
    pub name: String,
    pub filename: String,
    pub integrity: String,
    pub urls: Vec<String>,
}

impl DistFileEntry {
    pub fn try_new(source: &PackageDistSource) -> Result<Self> {
        Ok(Self {
            name: file_name_to_name(&source.filename),
            filename: source.filename.clone(),
            integrity: source.compute_integrity()?,
            urls: source.urls.iter().map(|url| url.to_string()).collect(),
        })
    }
}

/// Escapes a string so that it is safe to be embedded in a Starlark literal
/// string quoted with double-quotes (`"`).
///
/// Use this function with [`Tera::set_escape_fn`] to generate Starlark files.
pub fn escape_starlark_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_starlark_string() -> Result<()> {
        assert_eq!("", escape_starlark_string(""));
        assert_eq!("123", escape_starlark_string("123"));
        assert_eq!("abc", escape_starlark_string("abc"));
        assert_eq!(r#"\"foo\""#, escape_starlark_string(r#""foo""#));
        assert_eq!(r#"foo\\bar"#, escape_starlark_string(r#"foo\bar"#));
        Ok(())
    }
}
