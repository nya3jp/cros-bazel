// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Error, Result};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::multispace0,
    combinator::{eof, map, map_res, opt, verify},
    sequence::{preceded, tuple},
    IResult,
};
use url::Url;

use crate::dependency::{
    uri::{UriAtomDependency, UriDependency},
    CompositeDependency, Dependency,
};

use super::{DependencyParser, DependencyParserCommon};

/// Implements the URI dependency expression parser.
pub struct UriDependencyParser {}

impl<'i> DependencyParserCommon<'i, UriAtomDependency> for UriDependencyParser {
    fn new_all_of(children: Vec<UriDependency>) -> UriDependency {
        Dependency::new_composite(CompositeDependency::AllOf { children })
    }

    fn expression(input: &str) -> IResult<&str, UriDependency> {
        let (input, _) = multispace0(input)?;
        alt((
            map(Self::uri, |(url, filename)| {
                Dependency::Leaf(UriAtomDependency::Uri(url, filename.map(|s| s.to_owned())))
            }),
            map(Self::filename, |filename| {
                Dependency::Leaf(UriAtomDependency::Filename(filename.to_owned()))
            }),
            Self::all_of,
            Self::any_of,
            Self::use_conditional,
        ))(input)
    }
}

impl UriDependencyParser {
    fn uri(input: &str) -> IResult<&str, (Url, Option<&str>)> {
        let (input, url) = map_res(take_till(char::is_whitespace), Url::parse)(input)?;
        let (input, filename) = opt(preceded(
            tuple((multispace0, tag("->"), multispace0)),
            Self::filename,
        ))(input)?;
        Ok((input, (url, filename)))
    }

    fn filename(input: &str) -> IResult<&str, &str> {
        verify(take_till(char::is_whitespace), |s: &str| !s.contains('/'))(input)
    }

    fn full(input: &str) -> IResult<&str, UriDependency> {
        let (input, children) = Self::expression_list(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = eof(input)?;
        Ok((input, Self::new_all_of(children)))
    }
}

impl DependencyParser<UriDependency> for UriDependencyParser {
    type Err = Error;

    fn parse(input: &str) -> Result<UriDependency> {
        let (_, deps) = UriDependencyParser::full(input).map_err(|err| err.to_owned())?;
        Ok(deps)
    }
}
