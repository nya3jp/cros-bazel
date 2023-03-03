// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Error, Result};
use std::str::FromStr;

mod eval;
mod parser;

use crate::data::UseMap;

use self::parser::expression;

/// A simple bash command
#[derive(Clone, Debug, Eq, PartialEq)]
struct SimpleCommand {
    tokens: Vec<String>,
}

/// An AND-OR list is a sequence of one or more pipelines separated by the
/// operators "&&" and "||".
///
/// The operators "&&" and "||" shall have equal precedence and shall be
/// evaluated with left associativity. For example, both of the following
/// commands write solely bar to standard output:
/// ```bash
/// false && echo foo || echo bar
/// true || echo foo && echo bar
/// ```
/// Source: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_09_01
/// section 2.9.3 Lists.
#[derive(Clone, Debug, Eq, PartialEq)]
struct AndOrList {
    /// The first command in the list.
    ///
    /// Its return value will be used when evaluating the remaining `ops`.
    initial: SimpleCommand,

    /// list of && and || operators and commands.
    ops: Vec<AndOrListItem>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum AndOrListItem {
    /// The command will only be ran if the previous command returned success.
    AndOp(SimpleCommand),
    /// The command will only be ran if the previous command returned a failure.
    OrOp(SimpleCommand),
}

/// Represents a simple bash expression.
///
/// This struct only handles the bare minimum to support parsing the
/// CROS_WORKON_OPTIONAL_CHECKOUT expressions.
///
/// i.e., `use foo && use bar`
///
/// It doesn't handle string substitutions, redirections, arithmetic,
/// compound lists, loops, etc.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BashExpr {
    and_or_list: AndOrList,
}

impl FromStr for BashExpr {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (_, deps) = expression(input).map_err(|err| err.to_owned())?;
        Ok(deps)
    }
}

impl BashExpr {
    pub fn eval(&self, map: &UseMap) -> Result<bool> {
        self::eval::eval(self, map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_true() -> Result<()> {
        let expr = BashExpr::from_str("true")?;

        assert!(expr.eval(&UseMap::default())?, "expr = {:?}", expr);

        Ok(())
    }

    #[test]
    fn test_parse_false() -> Result<()> {
        let expr = BashExpr::from_str("false")?;

        assert!(!expr.eval(&UseMap::default())?, "expr = {:?}", expr);

        Ok(())
    }

    #[test]
    fn test_parse_unknown_command() -> Result<()> {
        let expr = BashExpr::from_str("echo hello world")?;

        assert!(expr.eval(&UseMap::default()).is_err(), "expr = {:?}", expr);

        Ok(())
    }

    #[test]
    fn test_parse_use_true() -> Result<()> {
        let expr = BashExpr::from_str("use foo")?;
        let map = UseMap::from([("foo".to_owned(), true)]);

        assert!(expr.eval(&map)?, "expr = {:?}", expr);

        Ok(())
    }

    #[test]
    fn test_parse_use_false() -> Result<()> {
        let expr = BashExpr::from_str("use foo")?;
        let map = UseMap::from([("foo".to_owned(), false)]);

        assert!(!expr.eval(&map)?, "expr = {:?}", expr);

        Ok(())
    }

    #[test]
    fn test_parse_use_not_true() -> Result<()> {
        let expr = BashExpr::from_str("use !foo")?;
        let map = UseMap::from([("foo".to_owned(), false)]);

        assert!(expr.eval(&map)?, "expr = {:?}", expr);

        Ok(())
    }

    #[test]
    fn test_parse_use_not_false() -> Result<()> {
        let expr = BashExpr::from_str("use !foo")?;
        let map = UseMap::from([("foo".to_owned(), true)]);

        assert!(!expr.eval(&map)?, "expr = {:?}", expr);

        Ok(())
    }


    #[test]
    fn test_parse_use_invalid() -> Result<()> {
        let expr = BashExpr::from_str("use foo bar")?;
        let map = UseMap::from([]);

        assert!(expr.eval(&map).is_err(), "expr = {:?}", expr);

        Ok(())
    }

    #[test]
    fn test_parse_and_true() -> Result<()> {
        let expr = BashExpr::from_str("use foo && use bar")?;
        let map = UseMap::from([("foo".to_owned(), true), ("bar".to_owned(), true)]);

        assert!(expr.eval(&map)?, "expr = {:?}", expr);

        Ok(())
    }

    #[test]
    fn test_parse_and_false() -> Result<()> {
        let expr = BashExpr::from_str("use foo && use bar")?;
        let map = UseMap::from([("foo".to_owned(), true), ("bar".to_owned(), false)]);

        assert!(!expr.eval(&map)?, "expr = {:?}", expr);

        Ok(())
    }

    #[test]
    fn test_parse_or_true() -> Result<()> {
        let expr = BashExpr::from_str("use foo || use bar")?;
        let map = UseMap::from([("foo".to_owned(), true), ("bar".to_owned(), true)]);

        assert!(expr.eval(&map)?, "expr = {:?}", expr);

        Ok(())
    }

    #[test]
    fn test_parse_or_true_1() -> Result<()> {
        let expr = BashExpr::from_str("use foo || use bar")?;
        let map = UseMap::from([("foo".to_owned(), false), ("bar".to_owned(), true)]);

        assert!(expr.eval(&map)?, "expr = {:?}", expr);

        Ok(())
    }

    #[test]
    fn test_parse_or_true_2() -> Result<()> {
        let expr = BashExpr::from_str("use foo || use bar")?;
        let map = UseMap::from([("foo".to_owned(), true), ("bar".to_owned(), false)]);

        assert!(expr.eval(&map)?, "expr = {:?}", expr);

        Ok(())
    }

    #[test]
    fn test_parse_or_false() -> Result<()> {
        let expr = BashExpr::from_str("use foo || use bar")?;
        let map = UseMap::from([("foo".to_owned(), false), ("bar".to_owned(), false)]);

        assert!(!expr.eval(&map)?, "expr = {:?}", expr);

        Ok(())
    }

    #[test]
    fn test_parse_complex() -> Result<()> {
        let expr = BashExpr::from_str("use foo && use bar || use baz")?;

        {
            let map = UseMap::from([("foo".to_owned(), true), ("bar".to_owned(), true)]);
            assert!(expr.eval(&map)?, "expr = {:?}", expr);
        }

        {
            let map = UseMap::from([("baz".to_owned(), true)]);
            assert!(expr.eval(&map)?, "expr = {:?}", expr);
        }

        {
            let map = UseMap::from([
                ("foo".to_owned(), true),
                ("bar".to_owned(), false),
                ("baz".to_owned(), true),
            ]);
            assert!(expr.eval(&map)?, "expr = {:?}", expr);
        }

        {
            let map = UseMap::from([
                ("foo".to_owned(), true),
                ("bar".to_owned(), false),
                ("baz".to_owned(), false),
            ]);
            assert!(!expr.eval(&map)?, "expr = {:?}", expr);
        }

        {
            let map = UseMap::from([
                ("foo".to_owned(), false),
                ("bar".to_owned(), true),
                ("baz".to_owned(), false),
            ]);
            assert!(!expr.eval(&map)?, "expr = {:?}", expr);
        }

        Ok(())
    }
}
