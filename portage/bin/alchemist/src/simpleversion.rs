// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::fmt;
use std::str::FromStr;

use nom::character::complete::{alpha1, digit1};

use nom::combinator::map;
use nom::sequence::pair;
use nom::{branch::alt, bytes::complete::take_while, combinator::eof, multi::many0, IResult};

use anyhow::{Error, Result};

#[derive(Debug, PartialEq, Eq)]
pub enum VersionComponent {
    Component(String),
    Separator(String),
}

/// Defines the Version strings (for manipulation API)
///
/// https://mgorny.pl/articles/the-ultimate-guide-to-eapi-7.html#version-strings-for-manipulation
#[derive(Debug, PartialEq, Eq)]
pub struct VersionComponents {
    pub components: Vec<VersionComponent>,
}

fn digit_component(input: &str) -> IResult<&str, VersionComponent> {
    map(digit1, |s: &str| VersionComponent::Component(s.to_string()))(input)
}

fn alpha_component(input: &str) -> IResult<&str, VersionComponent> {
    map(alpha1, |s: &str| VersionComponent::Component(s.to_string()))(input)
}

fn seperator_component(input: &str) -> IResult<&str, VersionComponent> {
    map(
        take_while(|c: char| !c.is_ascii_alphanumeric()),
        |s: &str| VersionComponent::Separator(s.to_string()),
    )(input)
}

fn expression(input: &str) -> IResult<&str, VersionComponents> {
    let mut parts = Vec::new();

    let (input, part) = seperator_component(input)?;
    if let VersionComponent::Separator(ref s) = part {
        if !s.is_empty() {
            parts.push(part);
        }
    }

    let (input, results) = many0(pair(
        alt((digit_component, alpha_component)),
        seperator_component,
    ))(input)?;

    let size = results.len();
    for (idx, (component, separator)) in results.into_iter().enumerate() {
        parts.push(component);

        // Avoid pushing an empty separator at the end
        if let VersionComponent::Separator(ref s) = separator {
            if idx != size - 1 || !s.is_empty() {
                parts.push(separator);
            }
        }
    }

    let (input, _) = eof(input)?;

    Ok((input, VersionComponents { components: parts }))
}

impl FromStr for VersionComponents {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (_, version) = expression(input).map_err(|err| err.to_owned())?;
        Ok(version)
    }
}

impl fmt::Display for VersionComponents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for component in &self.components {
            match component {
                VersionComponent::Component(c) => write!(f, "{}", c)?,
                VersionComponent::Separator(s) => write!(f, "{}", s)?,
            };
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(s: &str) -> VersionComponent {
        VersionComponent::Separator(s.to_string())
    }

    fn c(c: &str) -> VersionComponent {
        VersionComponent::Component(c.to_string())
    }

    #[test]
    fn test_empty() -> Result<()> {
        let actual: VersionComponents = "".parse()?;
        let expected: Vec<VersionComponent> = vec![];

        assert_eq!(expected, actual.components);
        assert_eq!("", format!("{}", actual));

        Ok(())
    }

    #[test]
    fn test_table() -> Result<()> {
        /*
        Taken from https://mgorny.pl/articles/the-ultimate-guide-to-eapi-7.html#version-strings-for-manipulation

        Type            s   c   s   c   s   c   s   c       s   c
        Index           0   1   1   2   2   3   3   4       4   5
        1.2.3               1   .   2   .   3
        1.2b_alpha4         1   .   2       b   _   alpha       4
        2Ab9s               2       Ab      9       s
        A.4.                A   .   4   .
        .11.            .   11  .

        */
        let table = [
            ("1.2.3", vec![c("1"), s("."), c("2"), s("."), c("3")]),
            (
                "1.2b_alpha4",
                vec![
                    c("1"),
                    s("."),
                    c("2"),
                    s(""),
                    c("b"),
                    s("_"),
                    c("alpha"),
                    s(""),
                    c("4"),
                ],
            ),
            (
                "2Ab9s",
                vec![c("2"), s(""), c("Ab"), s(""), c("9"), s(""), c("s")],
            ),
            ("A.4.", vec![c("A"), s("."), c("4"), s(".")]),
            (".11.", vec![s("."), c("11"), s(".")]),
        ];

        for (input, expected) in table {
            let actual: VersionComponents = input.parse()?;

            assert_eq!(expected, actual.components);
            assert_eq!(input, format!("{}", actual));
        }

        Ok(())
    }
}
