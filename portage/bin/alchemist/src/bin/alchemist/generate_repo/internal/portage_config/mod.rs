// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alchemist::{
    config::compiler::ProfileCompiler,
    fakechroot::{host_config_file_ops, target_config_file_ops, target_host_config_file_ops},
    fileops::FileOps,
    path::join_absolute,
};
use anyhow::{bail, ensure, Context, Result};
use lazy_static::lazy_static;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{borrow::Cow, fs::create_dir_all, path::Path};
use tera::Tera;

use crate::generate_repo::{common::AUTOGENERATE_NOTICE, TargetData};

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
    sysroot: &'a Path,
    ops: Vec<FileOps>,
    out: &'a Path,
) -> Result<PortageConfigTemplateContext<'a>> {
    let mut files = Vec::new();
    let mut symlinks = Vec::new();

    for op in ops {
        match op {
            FileOps::Symlink { source, target } => symlinks.push(SymlinkContext {
                source: source.into(),
                target: join_absolute(sysroot, &target)?.into(),
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
                    dest: join_absolute(sysroot, &path)?.into(),
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

/// Generates the native-root (/) config for the host board.
pub fn generate_host_portage_config(host: &TargetData, out: &Path) -> Result<()> {
    let out = out.join("host");

    create_dir_all(&out).with_context(|| format!("mkdir -p {out:?}"))?;

    let sysroot = Path::new("/");

    let compiler = ProfileCompiler::new(&host.config, &host.sysroot).strip_sysroot(true);

    let configs = vec![
        // This is the original profile that will be replaced by a "lite" profile.
        // None of the settings are used when cross-root compiling.
        file_ops_to_context("orig", sysroot, host_config_file_ops(None), &out)?,
        file_ops_to_context(
            "lite",
            sysroot,
            compiler.generate_lite_portage_config()?,
            &out,
        )?,
        file_ops_to_context("host", sysroot, compiler.generate_portage_config()?, &out)?,
        file_ops_to_context(
            "full",
            sysroot,
            host_config_file_ops(Some(&host.profile_path)),
            &out,
        )?,
    ];

    generate_build_file(configs, &out)
}

/// Generates the cross-root (/build/amd64-host) config for the host board.
pub fn generate_target_host_portage_config(host: &TargetData, out: &Path) -> Result<()> {
    let out = out.join("target/host");

    create_dir_all(&out).with_context(|| format!("mkdir -p {out:?}"))?;

    let sysroot = Path::new("/build").join(&host.board);

    let compiler = ProfileCompiler::new(&host.config, &host.sysroot);

    let configs = vec![
        file_ops_to_context(
            "full",
            &sysroot,
            target_host_config_file_ops(
                &host.board,
                &host.profile_path,
                &host.repos.get_repos(),
                &host.toolchains,
            )?,
            &out,
        )?,
        file_ops_to_context("host", &sysroot, compiler.generate_portage_config()?, &out)?,
    ];

    generate_build_file(configs, &out)
}

/// Generates the cross-root config for the target.
pub fn generate_target_portage_config(target: &TargetData, prefix: &str, out: &Path) -> Result<()> {
    let out = out.join(prefix);

    let name = prefix.split('/').last().expect("valid prefix");

    create_dir_all(&out).with_context(|| format!("mkdir -p {out:?}"))?;

    let sysroot = Path::new("/build").join(&target.board);

    let compiler = ProfileCompiler::new(&target.config, &target.sysroot);

    let configs = vec![
        file_ops_to_context(
            "full",
            &sysroot,
            target_config_file_ops(
                &target.board,
                &target.profile_path,
                &target.repos.get_repos(),
                &target.toolchains,
                false,
            )?,
            &out,
        )?,
        file_ops_to_context(name, &sysroot, compiler.generate_portage_config()?, &out)?,
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
    target: Option<&TargetData>,
    out: &Path,
) -> Result<()> {
    let out = out.join("internal/portage-config");

    generate_host_portage_config(host, &out)?;
    generate_target_host_portage_config(host, &out)?;
    if let Some(target) = target {
        generate_target_portage_config(target, "target/board", &out)?;
    }

    Ok(())
}
