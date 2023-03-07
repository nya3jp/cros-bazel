// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    ffi::OsStr,
    fs::{create_dir, create_dir_all, File},
    io::Write,
    os::unix::fs::symlink,
    path::Path,
};

use alchemist::repository::RepositorySet;
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
        symlink(&original_file, &output_file).with_context(|| {
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
struct OverlayBuildTemplateContext<'a> {
    name: &'a str,
    mount_path: &'a Path,
}

fn generate_overlay_build_file(relative_dir: &Path, output_file: &Path) -> Result<()> {
    // We don't use `relative_dir` because chromiumos != chromiumos-overlay.
    let name = relative_dir
        .file_name()
        .expect("repository name")
        .to_str()
        .expect("valid name");
    let mount_path = Path::new("src").join(relative_dir);
    let context = OverlayBuildTemplateContext {
        name,
        mount_path: &mount_path,
    };

    // The chromiumos-overlay repo contains a pretty complex BUILD.bazel file.
    // Once the bashrc and patch files can be cleaned up hopefully we can
    // use the standard template.
    let template = if relative_dir.to_string_lossy() == "third_party/chromiumos-overlay" {
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

pub fn generate_internal_overlays(
    src_dir: &Path,
    repos: &RepositorySet,
    output_dir: &Path,
) -> Result<()> {
    let output_overlays_dir = output_dir.join("internal/overlays");
    repos
        .get_repos()
        .into_iter()
        .try_for_each(|repo| -> Result<()> {
            let relative_dir = repo.base_dir().strip_prefix("/mnt/host/source/src")?;
            let original_dir = src_dir.join(relative_dir);
            let output_dir = output_overlays_dir.join(relative_dir);

            create_dir_all(&output_dir)
                .with_context(|| format!("mkdir -p {}", output_dir.display()))?;

            generate_overlay_symlinks(&original_dir, &output_dir)?;

            generate_overlay_build_file(relative_dir, &output_dir.join("BUILD.bazel"))?;

            Ok(())
        })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_overlay_build_file_succeeds() -> Result<()> {
        // Templates in this module are loaded together,
        // so syntax errors in any of them will fail the test.
        let relative_dir = Path::new("third_party/chromiumos-overlay");
        let output_file = tempfile::NamedTempFile::new()?;
        generate_overlay_build_file(relative_dir, output_file.path())
    }
}
