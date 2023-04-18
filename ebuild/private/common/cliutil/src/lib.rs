// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Result};

pub fn split_key_value(spec: &str) -> Result<(&str, &str)> {
    let v: Vec<_> = spec.split('=').collect();
    if v.len() != 2 {
        bail!("invalid spec: {:?}", spec);
    }
    Ok((v[0], v[1]))
}
