// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use nom::character::complete::multispace1;
use nom_regex::lib::nom::multi::separated_list1;
use nom_regex::str::re_find;
use once_cell::sync::Lazy;
use regex::Regex;

use nom::{
    branch::alt, bytes::complete::tag, character::complete::multispace0, combinator::eof,
    multi::many0, sequence::preceded, IResult,
};

use crate::bash::expr::{AndOrList, AndOrListItem, BashExpr, SimpleCommand};

/// Matches a sequence of non-quoted characters.
static UNQUOTED_TOKEN: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^[^|&;<>()$`\\"'\s]+"#).unwrap());

fn unquoted_token(input: &str) -> IResult<&str, &str> {
    re_find(UNQUOTED_TOKEN.clone())(input)
}

fn cmd(input: &str) -> IResult<&str, SimpleCommand> {
    let (input, tokens) =
        preceded(multispace0, separated_list1(multispace1, unquoted_token))(input)?;
    let tokens = tokens.into_iter().map(|s| s.to_owned()).collect();

    Ok((input, SimpleCommand { tokens }))
}

fn and_expr(input: &str) -> IResult<&str, AndOrListItem> {
    let (input, _) = multispace0(input)?;

    let (input, cmd) = preceded(tag("&&"), cmd)(input)?;

    Ok((input, AndOrListItem::AndOp(cmd)))
}

fn or_expr(input: &str) -> IResult<&str, AndOrListItem> {
    let (input, _) = multispace0(input)?;

    let (input, cmd) = preceded(tag("||"), cmd)(input)?;

    Ok((input, AndOrListItem::OrOp(cmd)))
}

fn and_or_item_expr(input: &str) -> IResult<&str, AndOrListItem> {
    alt((or_expr, and_expr))(input)
}

fn and_or_list_expr(input: &str) -> IResult<&str, AndOrList> {
    let (input, cmd) = cmd(input)?;

    let (input, and_or_ops) = many0(and_or_item_expr)(input)?;

    Ok((
        input,
        AndOrList {
            initial: cmd,
            ops: and_or_ops,
        },
    ))
}

/// Implements the CROS_WORKON_OPTIONAL_CHECKOUT parser.
/// This is a bash expression that is evaluated at runtime to determine
/// if a project should be checked out.
pub fn expression(input: &str) -> IResult<&str, BashExpr> {
    let (input, and_or_list) = and_or_list_expr(input)?;

    let (input, _) = eof(input)?;

    let expr = BashExpr { and_or_list };

    Ok((input, expr))
}

#[cfg(test)]
mod tests {
    use crate::bash::expr::AndOrList;
    use anyhow::Result;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_parse_empty() -> Result<()> {
        let expr = BashExpr::from_str("");

        assert!(expr.is_err());

        Ok(())
    }

    #[test]
    fn test_parse_true() -> Result<()> {
        let expr = BashExpr::from_str("true")?;

        assert_eq!(
            BashExpr {
                and_or_list: AndOrList {
                    initial: SimpleCommand {
                        tokens: vec!["true".to_owned()],
                    },
                    ops: vec![],
                }
            },
            expr,
        );

        Ok(())
    }

    #[test]
    fn test_parse_false() -> Result<()> {
        let expr = BashExpr::from_str("false")?;

        assert_eq!(
            BashExpr {
                and_or_list: AndOrList {
                    initial: SimpleCommand {
                        tokens: vec!["false".to_owned()],
                    },
                    ops: vec![],
                }
            },
            expr,
        );

        Ok(())
    }

    #[test]
    fn test_parse_multiple_tokens() -> Result<()> {
        let expr = BashExpr::from_str("echo hello world")?;

        assert_eq!(
            BashExpr {
                and_or_list: AndOrList {
                    initial: SimpleCommand {
                        tokens: vec!["echo".to_owned(), "hello".to_owned(), "world".to_owned()],
                    },
                    ops: vec![],
                }
            },
            expr,
        );

        Ok(())
    }

    #[test]
    fn test_parse_and_or() -> Result<()> {
        let expr = BashExpr::from_str("false && echo foo || echo bar")?;

        assert_eq!(
            BashExpr {
                and_or_list: AndOrList {
                    initial: SimpleCommand {
                        tokens: vec!["false".to_owned()],
                    },
                    ops: vec![
                        AndOrListItem::AndOp(SimpleCommand {
                            tokens: vec!["echo".to_owned(), "foo".to_owned()],
                        }),
                        AndOrListItem::OrOp(SimpleCommand {
                            tokens: vec!["echo".to_owned(), "bar".to_owned()],
                        }),
                    ],
                }
            },
            expr,
        );

        Ok(())
    }
}
