// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};

use crate::alchemist::TargetData;
use crate::generate_repo::common::{escape_starlark_string, AUTOGENERATE_NOTICE};
use anyhow::Result;
use lazy_static::lazy_static;
use serde::Serialize;
use tera::Tera;

lazy_static! {
    static ref TEMPLATE: Tera = {
        let mut tera: Tera = Default::default();
        tera.add_raw_template(
            "sysroot.BUILD.bazel",
            include_str!("templates/sysroot.BUILD.bazel"),
        )
        .unwrap();
        tera.autoescape_on(vec![".bazel"]);
        tera.set_escape_fn(escape_starlark_string);
        tera
    };
}

#[derive(Serialize)]
struct BuildTemplateContext<'a> {
    target_board: &'a str,
}

pub fn generate_sysroot_build_file(target: &TargetData, out: &Path) -> Result<()> {
    let output_dir = out.join("internal/sysroot");
    create_dir_all(&output_dir)?;
    let output_file = output_dir.join("BUILD.bazel");

    let context = BuildTemplateContext {
        target_board: &target.board,
    };

    let mut file = File::create(&output_file)?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    TEMPLATE.render_to(
        "sysroot.BUILD.bazel",
        &tera::Context::from_serialize(context)?,
        file,
    )?;
    Ok(())
}
