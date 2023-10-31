// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::generate_repo::TargetData;
use alchemist::fakechroot::host_config_file_ops;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{borrow::Cow, fs::create_dir_all, path::Path};

use alchemist::fileops::FileOps;
use anyhow::{bail, ensure, Context, Result};
use lazy_static::lazy_static;
use serde::Serialize;
use tera::Tera;

use crate::generate_repo::common::AUTOGENERATE_NOTICE;

use super::sdk::profile_path;

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera: Tera = Default::default();
        tera.add_raw_template(
            "portage-config.BUILD.bazel",
            include_str!("templates/portage-config.BUILD.bazel"),
        )
        .unwrap();
        tera
    };
}

#[derive(Serialize)]
struct FileContext<'a> {
    src: Cow<'a, Path>,
    dest: Cow<'a, Path>,
}

#[derive(Serialize)]
struct SymlinkContext<'a> {
    source: Cow<'a, Path>,
    target: Cow<'a, Path>,
}

#[derive(Serialize)]
struct PortageConfigTemplateContext<'a> {
    name: Cow<'a, str>,
    files: Vec<FileContext<'a>>,
    symlinks: Vec<SymlinkContext<'a>>,
}

#[derive(Serialize)]
struct PortageConfigsTemplateContext<'a> {
    configs: Vec<PortageConfigTemplateContext<'a>>,
}

fn generate_build_file(
    configs: Vec<PortageConfigTemplateContext>,
    output_dir: &Path,
) -> Result<()> {
    let context = PortageConfigsTemplateContext { configs };

    let output_file = output_dir.join("BUILD.bazel");

    let mut file =
        File::create(&output_file).with_context(|| format!("file {}", output_file.display()))?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    TEMPLATES.render_to(
        "portage-config.BUILD.bazel",
        &tera::Context::from_serialize(context)?,
        file,
    )?;

    Ok(())
}

fn file_ops_to_context<'a>(
    name: &'a str,
    ops: Vec<FileOps>,
    out: &'a Path,
) -> Result<PortageConfigTemplateContext<'a>> {
    let mut files = Vec::new();
    let mut symlinks = Vec::new();

    for op in ops {
        match op {
            FileOps::Symlink { source, target } => symlinks.push(SymlinkContext {
                source: source.into(),
                target: target.into(),
            }),
            FileOps::PlainFile { path, content } => {
                let file_name = PathBuf::from(format!(
                    "{}.{}",
                    name,
                    path.file_name().expect("file name").to_str().unwrap()
                ));

                let file_path = out.join(&file_name);

                // Make sure we don't have file conflicts since we are flattening the hierarchy.
                ensure!(
                    !file_path.try_exists()?,
                    "File already exists: {file_path:?}"
                );

                std::fs::write(&file_path, content)
                    .with_context(|| format!("file {}", file_path.display()))?;

                files.push(FileContext {
                    src: file_name.into(),
                    dest: path.into(),
                })
            }
            alchemist::fileops::FileOps::Mkdir { .. } => bail!("mkdir is not supported"),
        }
    }

    Ok(PortageConfigTemplateContext {
        name: name.into(),
        files,
        symlinks,
    })
}

pub fn generate_host_portage_config(host: &TargetData, out: &Path) -> Result<()> {
    let out = out.join("host");

    create_dir_all(&out).with_context(|| format!("mkdir -p {out:?}"))?;

    let configs = vec![
        // This is the original profile that will be replaced by a "lite" profile.
        // None of the settings are used when cross-root compiling.
        file_ops_to_context("orig", host_config_file_ops(None), &out)?,
        file_ops_to_context(
            "full",
            host_config_file_ops(Some(profile_path(&host.repos, &host.profile).as_path())),
            &out,
        )?,
    ];

    generate_build_file(configs, &out)
}

/// Generates the portage config (i.e., man 5 portage).
///
/// This will generate the following folder structure:
///   * //internal/portage-config
///       * host
///       * target
///           * board
///           * host
pub fn generate_portage_config(
    host: &TargetData,
    _target: Option<&TargetData>,
    out: &Path,
) -> Result<()> {
    let out = out.join("internal/portage-config");

    generate_host_portage_config(host, &out)?;

    Ok(())
}
