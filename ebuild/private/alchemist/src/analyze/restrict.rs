// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;

use crate::{
    data::UseMap,
    dependency::{
        algorithm::{elide_use_conditions, parse_simplified_dependency, simplify},
        restrict::{RestrictAtom, RestrictDependency},
    },
    ebuild::PackageDetails,
};

fn parse_restricts(deps: RestrictDependency, use_map: &UseMap) -> Result<Vec<RestrictAtom>> {
    let deps = elide_use_conditions(deps, use_map).unwrap_or_default();
    let deps = simplify(deps);
    parse_simplified_dependency(deps)
}

/// Analyzes ebuild variables and returns [`RestrictAtom`]s declared in the
/// ebuild.
pub fn analyze_restricts(details: &PackageDetails) -> Result<Vec<RestrictAtom>> {
    let restrict = details.vars.get_scalar_or_default("RESTRICT")?;
    let deps = restrict.parse::<RestrictDependency>()?;
    parse_restricts(deps, &details.use_map)
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{HashMap, HashSet},
        path::PathBuf,
    };

    use crate::{
        bash::vars::{BashValue, BashVars},
        data::Slot,
    };

    use super::*;

    fn new_package(restrict: Option<BashValue>, use_map: UseMap) -> PackageDetails {
        let mut vars: HashMap<String, BashValue> = HashMap::new();
        if let Some(value) = restrict {
            vars.insert("RESTRICT".to_owned(), value);
        }
        PackageDetails {
            repo_name: "baz".to_owned(),
            package_name: "foo/bar".to_owned(),
            version: "1.0".parse().unwrap(),
            vars: BashVars::new(vars),
            slot: Slot::new("0"),
            use_map,
            accepted: true,
            stable: true,
            masked: false,
            ebuild_path: PathBuf::from("/path/to/some.ebuild"),
            inherited: HashSet::new(),
        }
    }

    #[test]
    fn empty() -> Result<()> {
        let details = new_package(None, UseMap::new());
        let restricts = analyze_restricts(&details)?;
        assert_eq!(Vec::<RestrictAtom>::new(), restricts);
        Ok(())
    }

    #[test]
    fn simple() -> Result<()> {
        let details = new_package(
            Some(BashValue::Scalar(
                "   mirror\n\tnetwork-sandbox \n".to_owned(),
            )),
            UseMap::new(),
        );
        let restricts = analyze_restricts(&details)?;
        assert_eq!(
            vec![RestrictAtom::Mirror, RestrictAtom::NetworkSandbox],
            restricts
        );
        Ok(())
    }

    #[test]
    fn use_conditional() -> Result<()> {
        let details = new_package(
            Some(BashValue::Scalar(
                "mirror? ( mirror )\nnetwork-sandbox? ( network-sandbox )\n".to_owned(),
            )),
            UseMap::from([
                ("mirror".to_owned(), false),
                ("network-sandbox".to_owned(), true),
            ]),
        );
        let restricts = analyze_restricts(&details)?;
        assert_eq!(vec![RestrictAtom::NetworkSandbox], restricts);
        Ok(())
    }
}
