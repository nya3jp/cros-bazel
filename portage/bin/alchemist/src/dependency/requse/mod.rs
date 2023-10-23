// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::fmt::Display;

use crate::data::UseMap;

use self::parser::RequiredUseDependencyParser;

use super::ComplexDependency;
use super::DependencyMeta;
use super::Predicate;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequiredUseDependencyMeta;

impl DependencyMeta for RequiredUseDependencyMeta {
    type Leaf = RequiredUseAtom;
    type Parser = RequiredUseDependencyParser;
}

mod parser;

/// Alias of Dependency specialized to package REQUIRED_USE options.
pub type RequiredUseDependency = ComplexDependency<RequiredUseDependencyMeta>;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RequiredUseAtom {
    name: String,
    expect: bool,
}

impl Display for RequiredUseAtom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.expect {
            write!(f, "!")?;
        }
        write!(f, "{}", self.name)
    }
}

impl Predicate<()> for RequiredUseAtom {
    fn matches(&self, source_use_map: &UseMap, _target: &()) -> bool {
        *source_use_map.get(&self.name).unwrap_or(&false) == self.expect
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::dependency::ThreeValuedPredicate;

    use super::*;

    #[test]
    fn test_empty() {
        let deps = RequiredUseDependency::from_str("").unwrap();
        assert_eq!(
            deps.matches(&UseMap::from_iter([("xxx".into(), false)]), &()),
            Some(true)
        );
    }

    #[test]
    fn test_all_of() {
        let deps = RequiredUseDependency::from_str("aaa !bbb").unwrap();
        assert_eq!(
            deps.matches(
                &UseMap::from_iter([("aaa".into(), true), ("bbb".into(), false)]),
                &()
            ),
            Some(true)
        );
        assert_eq!(
            deps.matches(
                &UseMap::from_iter([("aaa".into(), false), ("bbb".into(), false)]),
                &()
            ),
            Some(false)
        );
        assert_eq!(
            deps.matches(
                &UseMap::from_iter([("aaa".into(), true), ("bbb".into(), true)]),
                &()
            ),
            Some(false)
        );
        assert_eq!(
            deps.matches(
                &UseMap::from_iter([("aaa".into(), false), ("bbb".into(), true)]),
                &()
            ),
            Some(false)
        );
    }

    #[test]
    fn test_any_of() {
        let deps = RequiredUseDependency::from_str("|| ( aaa !bbb )").unwrap();
        assert_eq!(
            deps.matches(
                &UseMap::from_iter([("aaa".into(), true), ("bbb".into(), false)]),
                &()
            ),
            Some(true)
        );
        assert_eq!(
            deps.matches(
                &UseMap::from_iter([("aaa".into(), false), ("bbb".into(), false)]),
                &()
            ),
            Some(true)
        );
        assert_eq!(
            deps.matches(
                &UseMap::from_iter([("aaa".into(), true), ("bbb".into(), true)]),
                &()
            ),
            Some(true)
        );
        assert_eq!(
            deps.matches(
                &UseMap::from_iter([("aaa".into(), false), ("bbb".into(), true)]),
                &()
            ),
            Some(false)
        );
    }

    #[test]
    fn test_exactly_one_of() {
        let deps = RequiredUseDependency::from_str("^^ ( aaa !bbb ccc )").unwrap();
        assert_eq!(
            deps.matches(
                &UseMap::from_iter([
                    ("aaa".into(), false),
                    ("bbb".into(), false),
                    ("ccc".into(), false),
                ]),
                &()
            ),
            Some(true)
        );
        assert_eq!(
            deps.matches(
                &UseMap::from_iter([
                    ("aaa".into(), true),
                    ("bbb".into(), false),
                    ("ccc".into(), false),
                ]),
                &()
            ),
            Some(false)
        );
        assert_eq!(
            deps.matches(
                &UseMap::from_iter([
                    ("aaa".into(), false),
                    ("bbb".into(), true),
                    ("ccc".into(), false),
                ]),
                &()
            ),
            Some(false)
        );
        assert_eq!(
            deps.matches(
                &UseMap::from_iter([
                    ("aaa".into(), false),
                    ("bbb".into(), false),
                    ("ccc".into(), true),
                ]),
                &()
            ),
            Some(false)
        );
    }

    #[test]
    fn test_at_most_one_of() {
        let deps = RequiredUseDependency::from_str("?? ( aaa !bbb ccc )").unwrap();
        assert_eq!(
            deps.matches(
                &UseMap::from_iter([
                    ("aaa".into(), false),
                    ("bbb".into(), false),
                    ("ccc".into(), false),
                ]),
                &()
            ),
            Some(true)
        );
        assert_eq!(
            deps.matches(
                &UseMap::from_iter([
                    ("aaa".into(), true),
                    ("bbb".into(), false),
                    ("ccc".into(), false),
                ]),
                &()
            ),
            Some(false)
        );
        assert_eq!(
            deps.matches(
                &UseMap::from_iter([
                    ("aaa".into(), false),
                    ("bbb".into(), true),
                    ("ccc".into(), false),
                ]),
                &()
            ),
            Some(true)
        );
        assert_eq!(
            deps.matches(
                &UseMap::from_iter([
                    ("aaa".into(), false),
                    ("bbb".into(), false),
                    ("ccc".into(), true),
                ]),
                &()
            ),
            Some(false)
        );
    }
}
