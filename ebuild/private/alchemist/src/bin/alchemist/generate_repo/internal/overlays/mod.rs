// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::HashSet,
    ffi::OsStr,
    fs::{create_dir, create_dir_all, File},
    io::Write,
    os::unix::fs::symlink,
    path::Path,
};

use alchemist::{
    fakechroot::PathTranslator,
    repository::{Repository, RepositorySet},
};
use anyhow::{Context, Result};
use lazy_static::lazy_static;
use serde::Serialize;
use tera::Tera;
use walkdir::WalkDir;

use crate::generate_repo::common::AUTOGENERATE_NOTICE;

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera: Tera = Default::default();
        tera.add_raw_template(
            "overlays.BUILD.bazel",
            include_str!("templates/overlays.BUILD.bazel"),
        )
        .unwrap();
        tera.add_raw_template(
            "overlay.BUILD.bazel",
            include_str!("templates/overlay.BUILD.bazel"),
        )
        .unwrap();
        tera.add_raw_template(
            "chromiumos-overlay.BUILD.bazel",
            include_str!("templates/chromiumos-overlay.BUILD.bazel"),
        )
        .unwrap();
        tera
    };
}

/// Mirrors files in the original overlay to the output tree with symlinks.
///
/// This function skips creating symlinks for these files:
/// - `**/BUILD.bazel`: They should not exist in overlays and interferes with
///   `BUILD.bazel` files we will generate later.
/// - `metadata/md5-cache`: They're not consumed by alchemist, and we have too
///   many files under the directory.
fn generate_overlay_symlinks(original_dir: &Path, output_dir: &Path) -> Result<()> {
    let walk = WalkDir::new(original_dir)
        .min_depth(1)
        .into_iter()
        .filter_entry(|entry| {
            if entry.file_name() == OsStr::new("BUILD.bazel") {
                return false;
            }
            let relative_path = entry.path().strip_prefix(original_dir).unwrap();
            if relative_path == Path::new("metadata/md5-cache") {
                return false;
            }
            true
        });
    for entry in walk {
        let entry = entry?;
        let original_file = entry.path();
        let relative_path = original_file.strip_prefix(original_dir).unwrap();
        let output_file = output_dir.join(relative_path);
        if entry.file_type().is_dir() {
            create_dir(&output_file).with_context(|| format!("mkdir {}", output_file.display()))?;
            continue;
        }
        symlink(original_file, &output_file).with_context(|| {
            format!(
                "ln -s {} {}",
                original_file.display(),
                output_file.display()
            )
        })?;
    }
    Ok(())
}

#[derive(Serialize)]
struct OverlaysTemplateContext {
    overlay_sets: Vec<OverlaySetTemplateContext>,
}

#[derive(Serialize)]
struct OverlaySetTemplateContext {
    name: String,
    overlays: Vec<String>,
}

#[derive(Serialize)]
struct OverlayBuildTemplateContext<'a> {
    name: &'a str,
    mount_path: &'a Path,
}

/// Helper struct to easily compute unique overlays to generate.
#[derive(Eq, PartialEq, Hash)]
struct SimpleRepository<'a> {
    name: &'a str,
    base_dir: &'a Path,
}

impl<'a> From<&'a Repository> for SimpleRepository<'a> {
    fn from(repo: &'a Repository) -> Self {
        SimpleRepository {
            name: repo.name(),
            base_dir: repo.base_dir(),
        }
    }
}

fn generate_overlays_file(repo_sets: &[&RepositorySet], output_dir: &Path) -> Result<()> {
    let context = OverlaysTemplateContext {
        overlay_sets: repo_sets
            .iter()
            .map(|r| OverlaySetTemplateContext {
                name: r.primary().name().to_string(),
                overlays: r
                    .get_repos()
                    .iter()
                    .map(|r| format!("//internal/overlays/{}", r.name()))
                    .collect(),
            })
            .collect(),
    };

    let output_file = output_dir.join("BUILD.bazel");

    let mut file =
        File::create(&output_file).with_context(|| format!("file {}", &output_file.display()))?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    TEMPLATES.render_to(
        "overlays.BUILD.bazel",
        &tera::Context::from_serialize(context)?,
        file,
    )?;

    Ok(())
}

fn generate_overlay_build_file(repo: &SimpleRepository, output_file: &Path) -> Result<()> {
    let context = OverlayBuildTemplateContext {
        name: repo.name,
        mount_path: repo.base_dir.strip_prefix("/").unwrap_or(repo.base_dir),
    };

    // The chromiumos-overlay repo contains a pretty complex BUILD.bazel file.
    // Once the bashrc and patch files can be cleaned up hopefully we can
    // use the standard template.
    let template = if repo.name == "chromiumos" {
        "chromiumos-overlay.BUILD.bazel"
    } else {
        "overlay.BUILD.bazel"
    };

    let mut file =
        File::create(output_file).with_context(|| format!("file {}", output_file.display()))?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    TEMPLATES.render_to(template, &tera::Context::from_serialize(context)?, file)?;

    Ok(())
}

fn merge_repo_sets<'a>(repo_sets: &'a [&'a RepositorySet]) -> HashSet<SimpleRepository> {
    repo_sets
        .iter()
        .flat_map(|s| s.get_repos())
        .map(|r| r.into())
        .collect()
}

pub fn generate_internal_overlays(
    translator: &PathTranslator,
    repo_sets: &[&RepositorySet],
    output_dir: &Path,
) -> Result<()> {
    let output_overlays_dir = output_dir.join("internal/overlays");

    merge_repo_sets(repo_sets)
        .iter()
        .try_for_each(|repo| -> Result<()> {
            let original_dir = translator.to_outer(repo.base_dir)?;
            let output_dir = output_overlays_dir.join(repo.name);

            create_dir_all(&output_dir)
                .with_context(|| format!("mkdir -p {}", output_dir.display()))?;

            generate_overlay_symlinks(&original_dir, &output_dir)?;

            generate_overlay_build_file(repo, &output_dir.join("BUILD.bazel"))?;

            Ok(())
        })?;

    generate_overlays_file(repo_sets, &output_overlays_dir)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_overlay_build_file_succeeds() -> Result<()> {
        // Templates in this module are loaded together,
        // so syntax errors in any of them will fail the test.
        let repo = SimpleRepository {
            name: "chromiumos",
            base_dir: Path::new("/mnt/host/source/src/third_party/chromiumos-overlay"),
        };

        let output_file = tempfile::NamedTempFile::new()?;
        generate_overlay_build_file(&repo, output_file.path())
    }
}
