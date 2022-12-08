// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use std::collections::HashMap;

pub type BashVars = HashMap<String, BashValue>;

/// Represents a shell variable value in bash.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BashValue {
    Scalar(String),
    // TODO: Support array values.
    UnsupportedArray,
}

/// Parses an output of `set -o posix; set` from bash to create a list of
/// shell variable names and their values as [BashVars].
pub(crate) fn parse_set_output(output: &str) -> Result<BashVars> {
    let (_, vars) = parser::set_output(output).map_err(|err| err.to_owned())?;
    Ok(vars)
}

mod parser {
    use nom::{
        branch::alt,
        bytes::complete::{is_not, tag, take_while, take_while1},
        character::{
            complete::{anychar, none_of},
            is_alphabetic, is_alphanumeric,
        },
        combinator::{eof, opt},
        multi::{many0, many1},
        sequence::{delimited, preceded},
        IResult,
    };

    use super::*;

    struct Assignment {
        name: String,
        value: BashValue,
    }

    fn variable(input: &str) -> IResult<&str, String> {
        let (input, a) = take_while1(|c| is_alphabetic(c as u8) || c == '_')(input)?;
        let (input, b) = take_while(|c| is_alphanumeric(c as u8) || c == '_')(input)?;
        Ok((input, format!("{}{}", a, b)))
    }

    fn unquoted_char(input: &str) -> IResult<&str, char> {
        alt((
            // https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_02
            none_of("|&;<>()$`\\\"\' \t\n"),
            preceded(tag("\\"), anychar),
        ))(input)
    }

    fn unquoted(input: &str) -> IResult<&str, String> {
        let (input, ss) = many1(unquoted_char)(input)?;
        Ok((input, ss.into_iter().collect()))
    }

    fn single_quoted(input: &str) -> IResult<&str, String> {
        let (input, s) = delimited(tag("'"), take_while(|c| c != '\''), tag("'"))(input)?;
        Ok((input, s.to_owned()))
    }

    fn word(input: &str) -> IResult<&str, String> {
        let (input, units) = many1(alt((unquoted, single_quoted)))(input)?;
        Ok((input, units.concat()))
    }

    fn scalar_value(input: &str) -> IResult<&str, BashValue> {
        let (input, s) = opt(word)(input)?;
        Ok((input, BashValue::Scalar(s.unwrap_or_default())))
    }

    fn array_value(input: &str) -> IResult<&str, BashValue> {
        // TODO: Support array values.
        // For now, just skip to the end of the line.
        let (input, _) = tag("(")(input)?;
        let (input, _) = is_not("\n")(input)?;
        Ok((input, BashValue::UnsupportedArray))
    }

    fn value(input: &str) -> IResult<&str, BashValue> {
        alt((array_value, scalar_value))(input)
    }

    fn assignment(input: &str) -> IResult<&str, Assignment> {
        let (input, name) = variable(input)?;
        let (input, _) = tag("=")(input)?;
        let (input, value) = value(input)?;
        let (input, _) = tag("\n")(input)?;
        Ok((input, Assignment { name, value }))
    }

    pub(super) fn set_output(input: &str) -> IResult<&str, BashVars> {
        let (input, assignments) = many0(assignment)(input)?;
        let (input, _) = eof(input)?;

        let vars = BashVars::from_iter(assignments.into_iter().map(|a| (a.name, a.value)));
        Ok((input, vars))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_set_output_unquoted() -> Result<()> {
        let vars = parse_set_output("LANG=en_US.UTF-8\n")?;
        assert_eq!(
            vars,
            BashVars::from_iter([(
                "LANG".to_owned(),
                BashValue::Scalar("en_US.UTF-8".to_owned())
            ),])
        );
        Ok(())
    }

    #[test]
    fn test_parse_set_output_escaped() -> Result<()> {
        let vars = parse_set_output("IFS=\\$\n")?;
        assert_eq!(
            vars,
            BashVars::from_iter([("IFS".to_owned(), BashValue::Scalar("$".to_owned())),])
        );
        Ok(())
    }

    #[test]
    fn test_parse_set_output_single_quoted() -> Result<()> {
        let vars = parse_set_output("LESSOPEN='|/usr/bin/lesspipe %s'\n")?;
        assert_eq!(
            vars,
            BashVars::from_iter([(
                "LESSOPEN".to_owned(),
                BashValue::Scalar("|/usr/bin/lesspipe %s".to_owned())
            ),])
        );
        Ok(())
    }

    #[test]
    fn test_parse_set_output_empty() -> Result<()> {
        let vars = parse_set_output("EMPTY=\n")?;
        assert_eq!(
            vars,
            BashVars::from_iter([("EMPTY".to_owned(), BashValue::Scalar("".to_owned()))])
        );
        Ok(())
    }

    #[test]
    fn test_parse_set_output_empty_single_quote() -> Result<()> {
        let vars = parse_set_output("EMPTY=''\n")?;
        assert_eq!(
            vars,
            BashVars::from_iter([("EMPTY".to_owned(), BashValue::Scalar("".to_owned()))])
        );
        Ok(())
    }

    #[test]
    fn test_parse_set_output_continued() -> Result<()> {
        let vars = parse_set_output("IFS='\n'\n")?;
        assert_eq!(
            vars,
            BashVars::from_iter([("IFS".to_owned(), BashValue::Scalar("\n".to_owned()))])
        );
        Ok(())
    }

    #[test]
    fn test_parse_set_output_complex_word() -> Result<()> {
        let vars = parse_set_output("FOO=''\\'bar\\ 'baz'\\'''\n")?;
        assert_eq!(
            vars,
            BashVars::from_iter([("FOO".to_owned(), BashValue::Scalar("'bar baz'".to_owned()))])
        );
        Ok(())
    }

    #[test]
    fn test_parse_set_output_arrays() -> Result<()> {
        let vars = parse_set_output("ARRAY=([0]=\"foo\" [1]=\"bar\")\nEMPTY_ARRAY=()\n")?;
        assert_eq!(
            vars,
            BashVars::from_iter([
                ("ARRAY".to_owned(), BashValue::UnsupportedArray),
                ("EMPTY_ARRAY".to_owned(), BashValue::UnsupportedArray),
            ])
        );
        Ok(())
    }

    #[test]
    fn test_parse_set_output_multiple_assignments() -> Result<()> {
        let vars = parse_set_output("A=a\nB='b'\nC=c\n")?;
        assert_eq!(
            vars,
            BashVars::from_iter([
                ("A".to_owned(), BashValue::Scalar("a".to_owned())),
                ("B".to_owned(), BashValue::Scalar("b".to_owned())),
                ("C".to_owned(), BashValue::Scalar("c".to_owned())),
            ])
        );
        Ok(())
    }
}
