// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alchemist::{
    analyze::source::PackageDistSource, ebuild::PackageDetails, repository::RepositorySet,
};
use anyhow::Result;
use itertools::Itertools;
use serde::Serialize;

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
/// For packages we maintain, we can add explicit BDEPENDs on the
/// cross-compilers since we only support a handful of architectures:
///
///    BDEPEND="!cros_host? (
///        arm? ( cross-armv7a-cros-linux-gnueabihf/go )
///        arm64? ( cross-aarch64-cros-linux-gnu/go )
///        amd64? ( cross-x86_64-cros-linux-gnu/go )
///    )"
///
/// Ideally we can prune this list as much as possible.
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
    format!("//internal/overlays:{}", repo_set.name())
}

pub fn package_details_to_target_path(details: &PackageDetails, prefix: &str) -> String {
    format!(
        "//internal/packages/{}/{}/{}:{}",
        prefix,
        details.as_basic_data().repo_name,
        details.as_basic_data().package_name,
        details.as_basic_data().version
    )
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
    s.replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
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
