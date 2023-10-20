// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
    combinator::opt,
    multi::separated_list0,
    sequence::{delimited, pair, preceded},
    IResult,
};
use nom_regex::str::re_find;
use once_cell::sync::Lazy;
use regex::Regex;

use super::{CompositeDependency, Dependency, DependencyMeta};

/// Regular expression matching a valid USE flag name.
static USE_NAME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Za-z0-9][A-Za-z0-9+_@-]*").unwrap());

/// Provides a dependency expression parser.
pub trait DependencyParser {
    type Output;
    type Err;

    fn parse(input: &str) -> Result<Self::Output, Self::Err>;
}

/// Provides the common implementation of dependency expression parser.
pub trait DependencyParserCommon<'i, M: DependencyMeta> {
    fn new_all_of(children: Vec<Dependency<M>>) -> Dependency<M>;
    fn expression(input: &'i str) -> IResult<&'i str, Dependency<M>>;

    fn all_of(input: &'i str) -> IResult<&'i str, Dependency<M>> {
        let (input, children) = delimited(
            pair(tag("("), multispace1),
            |input| Self::expression_list(input),
            pair(multispace1, tag(")")),
        )(input)?;
        Ok((
            input,
            Dependency::new_composite(CompositeDependency::AllOf { children }),
        ))
    }

    fn any_of(input: &'i str) -> IResult<&'i str, Dependency<M>> {
        let (input, _) = tag("||")(input)?;
        let (input, _) = multispace1(input)?;
        let (input, children) = delimited(
            pair(tag("("), multispace1),
            |input| Self::expression_list(input),
            pair(multispace1, tag(")")),
        )(input)?;
        Ok((
            input,
            Dependency::new_composite(CompositeDependency::AnyOf { children }),
        ))
    }

    fn use_conditional(input: &'i str) -> IResult<&'i str, Dependency<M>> {
        let (input, negate) = opt(tag("!"))(input)?;
        let expect = negate.is_none();
        let (input, name) = Self::use_name(input)?;
        let (input, _) = tag("?")(input)?;
        let (input, _) = multispace1(input)?;
        let (input, children) = delimited(
            pair(tag("("), multispace1),
            |input| Self::expression_list(input),
            pair(multispace1, tag(")")),
        )(input)?;
        Ok((
            input,
            Dependency::new_composite(CompositeDependency::UseConditional {
                name: name.to_owned(),
                expect,
                child: Self::new_all_of(children),
            }),
        ))
    }

    fn expression_list(input: &'i str) -> IResult<&'i str, Vec<Dependency<M>>> {
        let (input, exprs) = preceded(
            multispace0,
            separated_list0(multispace1, |input| Self::expression(input)),
        )(input)?;
        let exprs = exprs.into_iter().collect_vec();
        Ok((input, exprs))
    }

    fn use_name(input: &'i str) -> IResult<&'i str, &'i str> {
        re_find(USE_NAME_RE.clone())(input)
    }
}
