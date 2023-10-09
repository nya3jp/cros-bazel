// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use itertools::Itertools;

use crate::alchemist::TargetData;

#[derive(clap::Args, Clone, Debug)]
pub struct Args {
    /// Environment variables to dump.
    vars: Option<Vec<String>>,
}

pub fn dump_profile_main(target: &TargetData, args: Args) -> Result<()> {
    let map = target.config.env();

    let iter = if let Some(vars) = &args.vars {
        vars.iter().collect_vec().into_iter()
    } else {
        map.keys().sorted()
    };

    for key in iter {
        println!(
            "{}=\"{}\"",
            key,
            map.get(key)
                .with_context(|| format!("Failed to find key: {key}"))?
        );
    }
    Ok(())
}
