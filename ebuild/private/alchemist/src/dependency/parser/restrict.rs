// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::cell::Cell;

use anyhow::{Error, Result};
use nom::{
    branch::alt,
    bytes::complete::take_while1,
    character::complete::multispace0,
    combinator::{eof, map_res},
    IResult,
};

use crate::dependency::{
    restrict::{RestrictAtom, RestrictDependency},
    CompositeDependency, Dependency,
};

use super::{DependencyParser, DependencyParserCommon};

/// Implements the RESTRICT dependency expression parser.
pub struct RestrictDependencyParser {}

impl<'i> DependencyParserCommon<'i, RestrictAtom> for RestrictDependencyParser {
    fn new_all_of(children: Vec<RestrictDependency>) -> RestrictDependency {
        Dependency::new_composite(CompositeDependency::AllOf { children })
    }

    fn expression(input: &str) -> IResult<&str, RestrictDependency> {
        let (input, _) = multispace0(input)?;
        alt((
            Self::all_of,
            Self::any_of,
            Self::use_conditional,
            Self::restrict,
        ))(input)
    }
}

impl RestrictDependencyParser {
    fn restrict(input: &str) -> IResult<&str, RestrictDependency> {
        let first = Cell::new(true);
        let (input, value) = map_res(
            take_while1(|c| {
                if first.get() {
                    first.set(false);
                    char::is_alphabetic(c)
                } else {
                    char::is_alphabetic(c) || c == '-'
                }
            }),
            |s: &str| s.parse::<RestrictAtom>(),
        )(input)?;

        Ok((input, Dependency::Leaf(value)))
    }

    fn full(input: &str) -> IResult<&str, RestrictDependency> {
        let (input, children) = Self::expression_list(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = eof(input)?;
        Ok((input, Self::new_all_of(children)))
    }
}

impl DependencyParser<RestrictDependency> for RestrictDependencyParser {
    type Err = Error;

    fn parse(input: &str) -> Result<RestrictDependency> {
        let (_, deps) = RestrictDependencyParser::full(input).map_err(|err| err.to_owned())?;
        Ok(deps)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_parse_empty() -> Result<()> {
        let deps = RestrictDependency::from_str("")?;
        assert!(
            matches!(deps.check_constant(), Some((true, _))),
            "deps = {}",
            deps
        );
        Ok(())
    }

    #[test]
    fn test_parse_whitespace() -> Result<()> {
        let deps = RestrictDependency::from_str(" \r \n \t ")?;
        assert!(
            matches!(deps.check_constant(), Some((true, _))),
            "deps = {}",
            deps
        );
        Ok(())
    }

    #[test]
    fn test_parse_atoms() -> Result<()> {
        let deps = RestrictDependency::from_str("network-sandbox mirror")?;
        assert_eq!(
            RestrictDependency::new_composite(CompositeDependency::AllOf {
                children: vec![
                    RestrictDependency::Leaf(RestrictAtom::NetworkSandbox),
                    RestrictDependency::Leaf(RestrictAtom::Mirror),
                ]
            }),
            deps
        );
        Ok(())
    }

    #[test]
    fn test_parse_negative_atom() -> Result<()> {
        let deps = RestrictDependency::from_str("-mirror");
        assert!(deps.is_err());
        Ok(())
    }

    #[test]
    fn test_parse_atoms_complex() -> Result<()> {
        let deps = RestrictDependency::from_str("!test? ( test )")?;
        assert_eq!(
            RestrictDependency::new_composite(CompositeDependency::AllOf {
                children: vec![RestrictDependency::new_composite(
                    CompositeDependency::UseConditional {
                        name: "test".to_owned(),
                        expect: false,
                        child: RestrictDependency::new_composite(CompositeDependency::AllOf {
                            children: vec![RestrictDependency::Leaf(RestrictAtom::Test)],
                        }),
                    }
                ),],
            }),
            deps
        );
        Ok(())
    }
}
