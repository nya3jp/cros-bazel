// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{fs::File, io::Write, path::Path};

use anyhow::Result;
use lazy_static::lazy_static;
use serde::Serialize;
use tera::{Context, Tera};
use tracing::instrument;

use super::common::AUTOGENERATE_NOTICE;

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera: Tera = Default::default();
        tera.add_raw_template("settings.bzl", include_str!("templates/settings.bzl"))
            .unwrap();
        tera
    };
}

#[derive(Serialize)]
struct SettingsTemplateContext {
    board: String,
}

#[instrument(skip_all)]
pub fn generate_settings_bzl(board: &str, out: &Path) -> Result<()> {
    let context = SettingsTemplateContext {
        board: board.to_owned(),
    };

    let mut file = File::create(out)?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    TEMPLATES.render_to("settings.bzl", &Context::from_serialize(context)?, file)?;

    Ok(())
}
