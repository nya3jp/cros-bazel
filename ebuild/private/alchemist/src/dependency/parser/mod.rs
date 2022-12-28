// Copyright 2022 The ChromiumOS Authors.
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

use super::{CompositeDependency, Dependency};

pub mod package;
pub mod uri;

/// Regular expression matching a valid USE flag name.
static USE_NAME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Za-z0-9][A-Za-z0-9+_@-]*").unwrap());

/// Trait to be implemented by leaf types of [`Dependency`] to look up the
/// type implementing the [`DependencyParser`] trait.
///
/// `Dependency::from_str` uses this trait to locate the right parser function.
pub trait DependencyParserType<L> {
    type Parser: DependencyParser<Dependency<L>>;
}

/// Provides a dependency expression parser for the type `D`.
///
/// `D` is typically `Dependency<L>` where `L` is a dependency leaf type.
pub trait DependencyParser<D> {
    type Err;

    fn parse(input: &str) -> Result<D, Self::Err>;
}

/// Provides the common implementation of dependency expression parser.
trait DependencyParserCommon<'i, L> {
    fn new_all_of(children: Vec<Dependency<L>>) -> Dependency<L>;
    fn expression(input: &'i str) -> IResult<&'i str, Dependency<L>>;

    fn all_of(input: &'i str) -> IResult<&'i str, Dependency<L>> {
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

    fn any_of(input: &'i str) -> IResult<&'i str, Dependency<L>> {
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

    fn use_conditional(input: &'i str) -> IResult<&'i str, Dependency<L>> {
        let (input, negate) = opt(tag("!"))(input)?;
        let expect = negate.is_none();
        let (input, name) = re_find(USE_NAME_RE.clone())(input)?;
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

    fn expression_list(input: &'i str) -> IResult<&'i str, Vec<Dependency<L>>> {
        let (input, exprs) = preceded(
            multispace0,
            separated_list0(multispace1, |input| Self::expression(input)),
        )(input)?;
        let exprs = exprs.into_iter().collect_vec();
        Ok((input, exprs))
    }
}
