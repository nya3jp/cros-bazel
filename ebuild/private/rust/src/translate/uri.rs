// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;

use crate::{
    data::UseMap,
    dependency::{
        algorithm::{elide_use_conditions, simplify},
        uri::{UriAtomDependency, UriDependency},
    },
};

use super::parse_simplified_dependency;

pub fn translate_uri_dependencies(
    deps: UriDependency,
    use_map: &UseMap,
) -> Result<Vec<UriAtomDependency>> {
    let deps = elide_use_conditions(deps, &use_map).unwrap_or_default();
    let deps = simplify(deps);
    parse_simplified_dependency(deps)
}
