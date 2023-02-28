// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::sync::Arc;

use alchemist::{
    analyze::{
        dependency::PackageDependencies,
        source::{PackageDistSource, PackageSources},
    },
    ebuild::PackageDetails,
};
use anyhow::Result;
use itertools::Itertools;
use serde::Serialize;

pub static CHROOT_SRC_DIR: &str = "/mnt/host/source/src";

pub static AUTOGENERATE_NOTICE: &str = "# AUTO-GENERATED FILE. DO NOT EDIT.\n\n";

static DEFAULT_MIRRORS: &[&str] = &[
    "https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/",
    "https://commondatastorage.googleapis.com/chromeos-localmirror/distfiles/",
];

static ALLOWED_MIRRORS: &[&str] = &[
    "https://commondatastorage.googleapis.com/",
    "https://storage.googleapis.com/",
];

fn file_name_to_repository_name(file_name: &str) -> String {
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
    format!("portage-dist_{}", escaped_file_name)
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
}

#[derive(Serialize)]
pub struct DistFileEntry {
    pub repository_name: String,
    pub filename: String,
    pub integrity: String,
    pub urls: Vec<String>,
}

impl DistFileEntry {
    pub fn try_new(source: &PackageDistSource) -> Result<Self> {
        let special_urls = source
            .urls
            .iter()
            .flat_map(|url| {
                let url_string = {
                    let url = {
                        // Fix duplicated slashes in URL paths.
                        let mut url = url.clone();
                        url.set_path(&url.path().replace("//", "/"));
                        url
                    };
                    // Convert gs:// URLs to https:// URLs.
                    if url.scheme() == "gs" {
                        format!(
                            "https://storage.googleapis.com/{}{}",
                            url.host_str().unwrap_or_default(),
                            url.path()
                        )
                    } else {
                        url.to_string()
                    }
                };
                // Keep allow-listed URLs only.
                if ALLOWED_MIRRORS
                    .iter()
                    .all(|prefix| !url_string.starts_with(prefix))
                {
                    return None;
                }
                Some(url_string)
            })
            .collect_vec();

        let urls = if !special_urls.is_empty() {
            special_urls
        } else {
            DEFAULT_MIRRORS
                .iter()
                .map(|prefix| format!("{}{}", prefix, source.filename))
                .collect_vec()
        };

        Ok(Self {
            repository_name: file_name_to_repository_name(&source.filename),
            filename: source.filename.clone(),
            integrity: source.compute_integrity()?,
            urls,
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
