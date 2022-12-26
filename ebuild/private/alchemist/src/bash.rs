// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use std::collections::HashMap;

pub type BashVars = HashMap<String, BashValue>;

/// Represents a shell variable value in bash.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BashValue {
    Scalar(String),
    IndexedArray(Vec<String>),
    AssociativeArray(HashMap<String, String>),
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
        bytes::complete::{tag, take, take_while, take_while1},
        character::{
            complete::{anychar, multispace0, multispace1, none_of},
            is_alphabetic, is_alphanumeric,
        },
        combinator::{eof, map, map_res, opt, verify},
        multi::{many0, many1, separated_list0},
        sequence::{delimited, pair, preceded},
        IResult,
    };

    use super::*;

    struct Assignment {
        name: String,
        value: BashValue,
    }

    struct ArrayEntry {
        key: String,
        value: String,
    }

    impl ArrayEntry {
        fn to_value(entries: Vec<ArrayEntry>) -> BashValue {
            // If all keys can be parsed as integers less than INDEX_LIMIT,
            // convert the map to an indexed array.
            const INDEX_LIMIT: usize = 1000;
            let is_indexed = entries
                .iter()
                .all(|entry| match entry.key.parse::<usize>() {
                    Ok(index) if index < INDEX_LIMIT => true,
                    _ => false,
                });
            if is_indexed {
                return BashValue::IndexedArray(entries.into_iter().fold(
                    Vec::new(),
                    |mut array, entry| {
                        let index = entry.key.parse::<usize>().unwrap();
                        if index >= array.len() {
                            array.resize(index + 1, String::new());
                        }
                        array[index] = entry.value;
                        array
                    },
                ));
            }
            return BashValue::AssociativeArray(
                entries
                    .into_iter()
                    .map(|entry| (entry.key, entry.value))
                    .collect(),
            );
        }
    }

    fn variable(input: &str) -> IResult<&str, String> {
        let (input, a) = take_while1(|c| is_alphabetic(c as u8) || c == '_')(input)?;
        let (input, b) = take_while(|c| is_alphanumeric(c as u8) || c == '_')(input)?;
        Ok((input, format!("{}{}", a, b)))
    }

    fn unquoted_char(input: &str) -> IResult<&str, char> {
        alt((
            // https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_02
            // Also assume `[` and `]` are quoted to easily parse arrays.
            none_of("|&;<>()$`\\\"\' \t\n[]"),
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

    fn double_quoted_char(input: &str) -> IResult<&str, char> {
        // For our purpose, we don't need to handle expansion ($ and `).
        // Just handle escapes.
        alt((preceded(tag("\\"), anychar), none_of("\"")))(input)
    }

    fn double_quoted(input: &str) -> IResult<&str, String> {
        let (input, ss) = delimited(tag("\""), many0(double_quoted_char), tag("\""))(input)?;
        Ok((input, ss.into_iter().collect()))
    }

    fn dollar_quoted_char(input: &str) -> IResult<&str, char> {
        alt((
            // Named escapes.
            map(tag("\\a"), |_| '\x07'),
            map(tag("\\b"), |_| '\x08'),
            map(tag("\\e"), |_| '\x1b'),
            map(tag("\\E"), |_| '\x1b'),
            map(tag("\\f"), |_| '\x0c'),
            map(tag("\\n"), |_| '\x0a'),
            map(tag("\\r"), |_| '\x0d'),
            map(tag("\\t"), |_| '\x09'),
            map(tag("\\v"), |_| '\x0b'),
            map(tag("\\\\"), |_| '\\'),
            map(tag("\\'"), |_| '\''),
            map(tag("\\\""), |_| '"'),
            map(tag("\\?"), |_| '?'),
            // \nnn where nnn is a 3-digit octal number.
            // It is the only general escape method bash uses in the set output
            // (e.g. \xHH is not used).
            map_res(preceded(tag("\\"), take(3usize)), |s| {
                u8::from_str_radix(s, 8).map(|b| b as char)
            }),
            verify(
                map(take(1usize), |s: &str| s.chars().next().unwrap()),
                |c| *c != '\'',
            ),
        ))(input)
    }

    fn dollar_quoted(input: &str) -> IResult<&str, String> {
        let (input, ss) = delimited(tag("$'"), many0(dollar_quoted_char), tag("'"))(input)?;
        Ok((input, ss.into_iter().collect()))
    }

    fn word(input: &str) -> IResult<&str, String> {
        let (input, units) =
            many1(alt((unquoted, single_quoted, double_quoted, dollar_quoted)))(input)?;
        Ok((input, units.concat()))
    }

    fn scalar_value(input: &str) -> IResult<&str, BashValue> {
        let (input, s) = opt(word)(input)?;
        Ok((input, BashValue::Scalar(s.unwrap_or_default())))
    }

    fn array_entry(input: &str) -> IResult<&str, ArrayEntry> {
        let (input, key) = delimited(tag("["), word, tag("]="))(input)?;
        let (input, value) = word(input)?;
        Ok((input, ArrayEntry { key, value }))
    }

    fn array_value(input: &str) -> IResult<&str, BashValue> {
        let (input, entries) = delimited(
            pair(tag("("), multispace0),
            separated_list0(multispace1, array_entry),
            pair(multispace0, tag(")")),
        )(input)?;
        Ok((input, ArrayEntry::to_value(entries)))
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
    use anyhow::bail;

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
    fn test_parse_set_output_unquoted_escaped() -> Result<()> {
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
    fn test_parse_set_output_double_quoted() -> Result<()> {
        let vars = parse_set_output("LESSOPEN=\"|/usr/bin/lesspipe %s\"\n")?;
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
    fn test_parse_set_output_double_quoted_escaped() -> Result<()> {
        let vars = parse_set_output("IFS=\"\\$\"\n")?;
        assert_eq!(
            vars,
            BashVars::from_iter([("IFS".to_owned(), BashValue::Scalar("$".to_owned())),])
        );
        Ok(())
    }

    #[test]
    fn test_parse_set_output_dollar_quoted() -> Result<()> {
        let vars = parse_set_output(
            r#"TEXT=$'foo\a\b\e\E\f\n\r\t\v\\\'\"\?\001bar'
"#,
        )?;
        assert_eq!(
            vars,
            BashVars::from_iter([(
                "TEXT".to_owned(),
                BashValue::Scalar(
                    "foo\x07\x08\x1b\x1b\x0c\x0a\x0d\x09\x0b\\'\"?\x01bar".to_owned()
                )
            ),])
        );
        Ok(())
    }

    #[test]
    fn test_parse_set_output_verbatim_utf8() -> Result<()> {
        let vars = parse_set_output("TEXT=🐈'🐈'\"🐈\"$'🐈'\n")?;
        assert_eq!(
            vars,
            BashVars::from_iter([("TEXT".to_owned(), BashValue::Scalar("🐈🐈🐈🐈".to_owned())),])
        );
        Ok(())
    }

    // TODO: Enable this test after supporting escaped UTF-8.
    #[ignore]
    #[test]
    fn test_parse_set_output_dollar_quoted_utf8() -> Result<()> {
        let vars = parse_set_output(r#"TEXT=$'\360\237\220\210'\n"#)?;
        assert_eq!(
            vars,
            BashVars::from_iter([("TEXT".to_owned(), BashValue::Scalar("🐈".to_owned())),])
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
    fn test_parse_set_output_empty_arrays() -> Result<()> {
        let vars = parse_set_output("ARRAY=()\n")?;
        assert_eq!(
            vars,
            BashVars::from_iter([("ARRAY".to_owned(), BashValue::IndexedArray(vec![])),])
        );
        Ok(())
    }

    #[test]
    fn test_parse_set_output_indexed_arrays() -> Result<()> {
        let vars = parse_set_output("ARRAY=([0]=\"foo\" [1]=\"bar\")\n")?;
        assert_eq!(
            vars,
            BashVars::from_iter([(
                "ARRAY".to_owned(),
                BashValue::IndexedArray(vec!["foo".to_owned(), "bar".to_owned()])
            ),])
        );
        Ok(())
    }

    #[test]
    fn test_parse_set_output_indexed_arrays_sparse() -> Result<()> {
        let vars = parse_set_output("ARRAY=([1]=\"foo\" [4]=\"bar\")\n")?;
        assert_eq!(
            vars,
            BashVars::from_iter([(
                "ARRAY".to_owned(),
                BashValue::IndexedArray(vec![
                    "".to_owned(),
                    "foo".to_owned(),
                    "".to_owned(),
                    "".to_owned(),
                    "bar".to_owned()
                ])
            ),])
        );
        Ok(())
    }

    #[test]
    fn test_parse_set_output_large_indexed_arrays() -> Result<()> {
        let vars = parse_set_output("ARRAY1=([999]=\"foo\")\nARRAY2=([1000]=\"foo\")\n")?;
        // ARRAY1 should be IndexedArray, whereas ARRAY2 should be AssociativeArray
        // since its key is too large.
        match &vars["ARRAY1"] {
            BashValue::IndexedArray(values) if values.len() == 1000 => {}
            other => bail!("ARRAY1 has unexpected value: {:?}", other),
        }
        match &vars["ARRAY2"] {
            BashValue::AssociativeArray(_) => {}
            other => bail!("ARRAY2 has unexpected value: {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn test_parse_set_output_associative_arrays() -> Result<()> {
        let vars = parse_set_output("ARRAY=([foo]=\"FOO\" [\"bar\"]='BAR')\n")?;
        assert_eq!(
            vars,
            BashVars::from_iter([(
                "ARRAY".to_owned(),
                BashValue::AssociativeArray(
                    [
                        ("foo".to_owned(), "FOO".to_owned()),
                        ("bar".to_owned(), "BAR".to_owned())
                    ]
                    .into()
                )
            ),])
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
