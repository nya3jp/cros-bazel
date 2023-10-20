// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
    combinator::{map, opt},
    multi::separated_list0,
    sequence::{delimited, pair, preceded},
    IResult,
};
use nom_regex::str::re_find;
use once_cell::sync::Lazy;
use regex::Regex;

use super::{ComplexCompositeDependency, CompositeDependency};

/// Regular expression matching a valid USE flag name.
static USE_NAME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Za-z0-9][A-Za-z0-9+_@-]*").unwrap());

/// Provides a dependency expression parser.
pub trait DependencyParser {
    type Output;
    type Err;

    fn parse(input: &str) -> Result<Self::Output, Self::Err>;
}

/// Implements the partial parser of dependency expressions.
///
/// This type is required when you call common parsing functions defined in this module.
pub trait PartialExpressionParser {
    type Output;

    /// Consumes an expression found on the beginning of the input.
    fn parse_expression(input: &str) -> IResult<&str, Self::Output>;
}

/// Consumes zero or more expressions found on the beginning of the input.
pub fn parse_expression_list<P: PartialExpressionParser>(
    input: &str,
) -> IResult<&str, Vec<P::Output>> {
    let (input, exprs) = preceded(
        multispace0,
        separated_list0(multispace1, |input| P::parse_expression(input)),
    )(input)?;
    let exprs = exprs.into_iter().collect_vec();
    Ok((input, exprs))
}

/// Consumes a group expression found on the beginning of the input.
///
/// A group expression is generalization of composite expressions, such as
/// all-of and any-of.
fn parse_group<'i, P: PartialExpressionParser>(
    input: &'i str,
    marker: Option<&str>,
) -> IResult<&'i str, Vec<P::Output>> {
    let input = if let Some(marker) = marker {
        let (input, _) = tag(marker)(input)?;
        let (input, _) = multispace1(input)?;
        input
    } else {
        input
    };
    let (input, children) = delimited(
        pair(tag("("), multispace1),
        |input| parse_expression_list::<P>(input),
        pair(multispace1, tag(")")),
    )(input)?;
    Ok((input, children))
}

/// Consumes a USE flag name found on the beginning of the input.
pub fn parse_use_name(input: &str) -> IResult<&str, &str> {
    re_find(USE_NAME_RE.clone())(input)
}

/// Result of [`parse_use_conditional`].
struct ParsedUseConditional<'i, D> {
    name: &'i str,
    expect: bool,
    children: Vec<D>,
}

/// Consumes a USE conditional expression found on the beginning of the input.
fn parse_use_conditional<P: PartialExpressionParser>(
    input: &str,
) -> IResult<&str, ParsedUseConditional<P::Output>> {
    let (input, negate) = opt(tag("!"))(input)?;
    let expect = negate.is_none();
    let (input, name) = parse_use_name(input)?;
    let (input, _) = tag("?")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, children) = delimited(
        pair(tag("("), multispace1),
        |input| parse_expression_list::<P>(input),
        pair(multispace1, tag(")")),
    )(input)?;
    Ok((
        input,
        ParsedUseConditional {
            name,
            expect,
            children,
        },
    ))
}

/// Consumes a composite expression found on the beginning of the input and
/// returns [`CompositeDependency`].
pub fn parse_composite<P: PartialExpressionParser>(
    input: &str,
) -> IResult<&str, CompositeDependency<P::Output>> {
    alt((
        map(
            |input| parse_group::<P>(input, None),
            |children| CompositeDependency::AllOf { children },
        ),
        map(
            |input| parse_group::<P>(input, Some("||")),
            |children| CompositeDependency::AnyOf { children },
        ),
        map(parse_use_conditional::<P>, |parsed| {
            CompositeDependency::UseConditional {
                name: parsed.name.to_string(),
                expect: parsed.expect,
                children: parsed.children,
            }
        }),
    ))(input)
}

/// Consumes a complex composite expression found on the beginning of the input
/// and returns [`ComplexCompositeDependency`].
pub fn parse_complex_composite<P: PartialExpressionParser>(
    input: &str,
) -> IResult<&str, ComplexCompositeDependency<P::Output>> {
    alt((
        map(
            |input| parse_group::<P>(input, None),
            |children| ComplexCompositeDependency::AllOf { children },
        ),
        map(
            |input| parse_group::<P>(input, Some("||")),
            |children| ComplexCompositeDependency::AnyOf { children },
        ),
        map(
            |input| parse_group::<P>(input, Some("^^")),
            |children| ComplexCompositeDependency::ExactlyOneOf { children },
        ),
        map(
            |input| parse_group::<P>(input, Some("??")),
            |children| ComplexCompositeDependency::AtMostOneOf { children },
        ),
        map(parse_use_conditional::<P>, |parsed| {
            ComplexCompositeDependency::UseConditional {
                name: parsed.name.to_string(),
                expect: parsed.expect,
                children: parsed.children,
            }
        }),
    ))(input)
}
