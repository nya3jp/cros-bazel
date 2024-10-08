// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Error, Result};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1},
    character::complete::multispace0,
    combinator::{eof, map, map_res, opt, verify},
    sequence::{preceded, tuple},
    IResult,
};
use url::Url;

use crate::dependency::{
    parser::{parse_composite, parse_expression_list, DependencyParser, PartialExpressionParser},
    uri::{UriAtomDependency, UriDependency},
    CompositeDependency, Dependency,
};

/// Implements the URI dependency expression parser.
pub struct UriDependencyParser;

impl PartialExpressionParser for UriDependencyParser {
    type Output = UriDependency;

    fn parse_expression(input: &str) -> IResult<&str, Self::Output> {
        let (input, _) = multispace0(input)?;
        alt((
            // Prefer matches with composite dependencies since URIs/filenames
            // consist of arbitrary characters.
            map(parse_composite::<Self>, Dependency::new_composite),
            map(Self::uri, |(url, filename)| {
                Dependency::Leaf(UriAtomDependency::Uri(url, filename.map(|s| s.to_owned())))
            }),
            map(Self::filename, |filename| {
                Dependency::Leaf(UriAtomDependency::Filename(filename.to_owned()))
            }),
        ))(input)
    }
}

impl UriDependencyParser {
    fn uri(input: &str) -> IResult<&str, (Url, Option<&str>)> {
        let (input, url) = map_res(take_till1(char::is_whitespace), Url::parse)(input)?;
        let (input, filename) = opt(preceded(
            tuple((multispace0, tag("->"), multispace0)),
            Self::filename,
        ))(input)?;
        Ok((input, (url, filename)))
    }

    fn filename(input: &str) -> IResult<&str, &str> {
        // Avoid matching with a closing parenthesis.
        verify(take_till1(char::is_whitespace), |s: &str| s != ")")(input)
    }

    fn full(input: &str) -> IResult<&str, UriDependency> {
        let (input, children) = parse_expression_list::<Self>(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = eof(input)?;
        Ok((
            input,
            Dependency::new_composite(CompositeDependency::AllOf { children }),
        ))
    }
}

impl DependencyParser for UriDependencyParser {
    type Output = UriDependency;
    type Err = Error;

    fn parse(input: &str) -> Result<Self::Output> {
        let (_, deps) = UriDependencyParser::full(input).map_err(|err| err.to_owned())?;
        Ok(deps)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_parse_empty() -> Result<()> {
        let deps = UriDependency::from_str("")?;
        assert!(
            matches!(deps.check_constant(), Some((true, _))),
            "deps = {}",
            deps
        );
        Ok(())
    }

    #[test]
    fn test_parse_whitespace() -> Result<()> {
        let deps = UriDependency::from_str(" \r \n \t ")?;
        assert!(
            matches!(deps.check_constant(), Some((true, _))),
            "deps = {}",
            deps
        );
        Ok(())
    }

    #[test]
    fn test_parse_atoms() -> Result<()> {
        let deps = UriDependency::from_str("https://www.google.com/robots.txt kitten.jpg")?;
        assert_eq!(
            UriDependency::new_composite(CompositeDependency::AllOf {
                children: vec![
                    UriDependency::Leaf(UriAtomDependency::Uri(
                        Url::parse("https://www.google.com/robots.txt")?,
                        None
                    )),
                    UriDependency::Leaf(UriAtomDependency::Filename("kitten.jpg".to_owned()))
                ]
            }),
            deps
        );
        Ok(())
    }

    #[test]
    fn test_parse_composite() -> Result<()> {
        let deps = UriDependency::from_str("|| ( foo? ( bar ) )")?;
        assert_eq!(
            UriDependency::new_composite(CompositeDependency::AllOf {
                children: vec![UriDependency::new_composite(CompositeDependency::AnyOf {
                    children: vec![UriDependency::new_composite(
                        CompositeDependency::UseConditional {
                            name: "foo".to_owned(),
                            expect: true,
                            children: vec![UriDependency::Leaf(UriAtomDependency::Filename(
                                "bar".to_owned()
                            ))],
                        }
                    ),],
                }),],
            }),
            deps
        );
        Ok(())
    }
}
