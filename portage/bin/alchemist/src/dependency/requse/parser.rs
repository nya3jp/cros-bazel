// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Error, Result};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::multispace0,
    combinator::{eof, map, opt},
    IResult,
};

use crate::dependency::{
    parser::{
        parse_complex_composite, parse_expression_list, parse_use_name, DependencyParser,
        PartialExpressionParser,
    },
    requse::{RequiredUseAtom, RequiredUseDependency},
    ComplexCompositeDependency, ComplexDependency,
};

/// Implements the REQUIRED_USE dependency expression parser.
pub struct RequiredUseDependencyParser;

impl PartialExpressionParser for RequiredUseDependencyParser {
    type Output = RequiredUseDependency;

    fn parse_expression(input: &str) -> IResult<&str, Self::Output> {
        let (input, _) = multispace0(input)?;
        alt((
            map(
                parse_complex_composite::<Self>,
                ComplexDependency::new_composite,
            ),
            Self::atom,
        ))(input)
    }
}

impl RequiredUseDependencyParser {
    fn atom(input: &str) -> IResult<&str, RequiredUseDependency> {
        let (input, negate) = opt(tag("!"))(input)?;
        let expect = negate.is_none();
        let (input, name) = parse_use_name(input)?;

        Ok((
            input,
            ComplexDependency::Leaf(RequiredUseAtom {
                name: name.to_string(),
                expect,
            }),
        ))
    }

    fn full(input: &str) -> IResult<&str, RequiredUseDependency> {
        let (input, children) = parse_expression_list::<Self>(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = eof(input)?;
        Ok((
            input,
            ComplexDependency::new_composite(ComplexCompositeDependency::AllOf { children }),
        ))
    }
}

impl DependencyParser for RequiredUseDependencyParser {
    type Output = RequiredUseDependency;
    type Err = Error;

    fn parse(input: &str) -> Result<Self::Output> {
        let (_, deps) = RequiredUseDependencyParser::full(input).map_err(|err| err.to_owned())?;
        Ok(deps)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_parse_empty() -> Result<()> {
        let deps = RequiredUseDependency::from_str("")?;
        assert_eq!(
            deps,
            ComplexDependency::new_composite(ComplexCompositeDependency::AllOf {
                children: Vec::new()
            }),
            "deps = {}",
            deps
        );
        Ok(())
    }

    #[test]
    fn test_parse_whitespace() -> Result<()> {
        let deps = RequiredUseDependency::from_str(" \r \n \t ")?;
        assert_eq!(
            deps,
            ComplexDependency::new_composite(ComplexCompositeDependency::AllOf {
                children: Vec::new()
            }),
            "deps = {}",
            deps
        );
        Ok(())
    }

    #[test]
    fn test_parse_atoms() -> Result<()> {
        let deps = RequiredUseDependency::from_str("static !test python")?;
        assert_eq!(
            deps,
            ComplexDependency::new_composite(ComplexCompositeDependency::AllOf {
                children: vec![
                    RequiredUseDependency::Leaf(RequiredUseAtom {
                        name: "static".into(),
                        expect: true
                    }),
                    RequiredUseDependency::Leaf(RequiredUseAtom {
                        name: "test".into(),
                        expect: false
                    }),
                    RequiredUseDependency::Leaf(RequiredUseAtom {
                        name: "python".into(),
                        expect: true
                    }),
                ]
            }),
            "deps = {}",
            deps
        );
        Ok(())
    }

    #[test]
    fn test_parse_complex() -> Result<()> {
        let deps = RequiredUseDependency::from_str("^^ ( aaa bbb ) ?? ( ccc ddd ) !test? ( eee )")?;
        assert_eq!(
            deps,
            ComplexDependency::new_composite(ComplexCompositeDependency::AllOf {
                children: vec![
                    ComplexDependency::new_composite(ComplexCompositeDependency::ExactlyOneOf {
                        children: vec![
                            RequiredUseDependency::Leaf(RequiredUseAtom {
                                name: "aaa".into(),
                                expect: true
                            }),
                            RequiredUseDependency::Leaf(RequiredUseAtom {
                                name: "bbb".into(),
                                expect: true
                            }),
                        ]
                    }),
                    ComplexDependency::new_composite(ComplexCompositeDependency::AtMostOneOf {
                        children: vec![
                            RequiredUseDependency::Leaf(RequiredUseAtom {
                                name: "ccc".into(),
                                expect: true
                            }),
                            RequiredUseDependency::Leaf(RequiredUseAtom {
                                name: "ddd".into(),
                                expect: true
                            }),
                        ]
                    }),
                    ComplexDependency::new_composite(ComplexCompositeDependency::UseConditional {
                        name: "test".into(),
                        expect: false,
                        children: vec![RequiredUseDependency::Leaf(RequiredUseAtom {
                            name: "eee".into(),
                            expect: true
                        }),]
                    }),
                ]
            }),
            "deps = {}",
            deps
        );
        Ok(())
    }
}
