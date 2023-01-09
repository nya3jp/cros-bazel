// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{cmp::Ordering, process::exit};

use anyhow::{anyhow, bail, Result};
use itertools::Itertools;
use version::Version;

fn main() -> Result<()> {
    let mut args = std::env::args().into_iter().skip(1).collect_vec();
    if args.len() == 2 {
        args.insert(0, std::env::var("PVR").unwrap_or_default());
    }
    let (lhs, op, rhs) = args
        .into_iter()
        .collect_tuple()
        .ok_or_else(|| anyhow!("Needs 2 or 3 arguments"))?;

    let lhs = Version::try_new(&lhs)?;
    let rhs = Version::try_new(&rhs)?;
    let ord = lhs.cmp(&rhs);

    let ok = match op.as_str() {
        "-eq" => ord == Ordering::Equal,
        "-ne" => ord != Ordering::Equal,
        "-gt" => ord == Ordering::Greater,
        "-ge" => ord != Ordering::Less,
        "-lt" => ord == Ordering::Less,
        "-le" => ord != Ordering::Greater,
        _ => bail!("Unsupported operator: {}", &op),
    };

    exit(if ok { 0 } else { 1 });
}
