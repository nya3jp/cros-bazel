// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    fs::{create_dir_all, File},
    io::Write,
    os::unix::fs::symlink,
    path::Path,
};

use alchemist::{
    fakechroot::PathTranslator,
    repository::{RepositorySetOperations, UnorderedRepositorySet},
};
use anyhow::{Context, Result};
use itertools::Itertools;
use lazy_static::lazy_static;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Serialize;
use tera::Tera;
use tracing::instrument;

use crate::{alchemist::TargetData, generate_repo::common::AUTOGENERATE_NOTICE};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera: Tera = Default::default();
        tera.add_raw_template(
            "bashrc.BUILD.bazel",
            include_str!("templates/bashrc.BUILD.bazel"),
        )
        .unwrap();
        tera
    };
}

/// symlinks the original bashrc to the output tree.
///
/// The target symlinks names are just the index of the bashrc. This is so we
/// can reuse the name of the file as the target name in the generated
/// BUILD.bazel file.
fn generate_bashrc_symlinks<'a>(
    output_dir: &'a Path,
    bashrcs: &'a [&'a Path],
    translator: &'a PathTranslator,
) -> Result<()> {
    for (i, bashrc) in bashrcs.iter().enumerate() {
        let output_file = output_dir.join(i.to_string());
        let real_file = translator.to_outer(bashrc)?;
        symlink(&real_file, &output_file)
            .with_context(|| format!("ln -s {} {}", real_file.display(), output_file.display()))?;
    }
    Ok(())
}

#[derive(Serialize)]
struct BashRcsTemplateContext<'a> {
    bashrcs: Vec<BashRcTemplateContext<'a>>,
}

#[derive(Serialize)]
struct BashRcTemplateContext<'a> {
    name: &'a str,
    src: usize,
    dest: &'a Path,
}

fn generate_bashrc_build_file(output_file: &Path, bashrcs: &[&Path]) -> Result<()> {
    let context = BashRcsTemplateContext {
        bashrcs: bashrcs
            .iter()
            .enumerate()
            .map(|(i, bashrc)| {
                Ok(BashRcTemplateContext {
                    name: bashrc
                        .file_name()
                        .with_context(|| format!("{bashrc:?} is missing file name"))?
                        .to_str()
                        .with_context(|| format!("{bashrc:?} is not a valid string"))?,
                    src: i,
                    dest: bashrc,
                })
            })
            .collect::<Result<_>>()?,
    };

    let mut file =
        File::create(output_file).with_context(|| format!("file {}", output_file.display()))?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    TEMPLATES.render_to(
        "bashrc.BUILD.bazel",
        &tera::Context::from_serialize(context)?,
        file,
    )?;

    Ok(())
}

#[instrument(skip_all)]
pub fn generate_internal_bashrcs(
    translator: &PathTranslator,
    host: &TargetData,
    target: Option<&TargetData>,
    output_dir: &Path,
) -> Result<()> {
    let output_overlays_dir = output_dir.join("internal/bashrcs");
    let merged_repo_set: UnorderedRepositorySet = host
        .repos
        .get_repos()
        .into_iter()
        .chain(target.map_or(vec![], |data| data.repos.get_repos()))
        .cloned()
        .collect();

    let profile_bashrcs = merged_repo_set.group_paths_by_repos(
        host.config
            .all_profile_bashrcs()
            .into_iter()
            .chain(target.map_or(vec![], |data| data.config.all_profile_bashrcs())),
    )?;

    let package_bashrcs = merged_repo_set.group_paths_by_repos(
        host.config
            .all_package_bashrcs()
            .into_iter()
            .chain(target.map_or(vec![], |data| data.config.all_package_bashrcs())),
    )?;

    let output_dirs_to_bashrcs = profile_bashrcs
        .into_iter()
        .chain(package_bashrcs)
        .flat_map(|(repo_name, bashrcs)| {
            let repo = merged_repo_set
                .get_repo_by_name(repo_name)
                .expect("repo to exist");

            bashrcs.into_iter().sorted().map(|full_path| {
                (
                    output_overlays_dir.join(repo.name()).join(
                        full_path
                            .parent()
                            .expect("bashrc to have a parent dir")
                            .strip_prefix(repo.profiles_dir())
                            .expect("bashrc to be contained in repo"),
                    ),
                    full_path,
                )
            })
        })
        .into_group_map();

    output_dirs_to_bashrcs.keys().sorted().try_for_each(|dir| {
        create_dir_all(dir).with_context(|| format!("mkdir -p {}", dir.display()))
    })?;

    output_dirs_to_bashrcs
        .par_iter()
        .try_for_each(|(output_dir, bashrcs)| {
            generate_bashrc_symlinks(output_dir, bashrcs, translator)?;

            generate_bashrc_build_file(&output_dir.join("BUILD.bazel"), bashrcs)
        })?;

    Ok(())
}
