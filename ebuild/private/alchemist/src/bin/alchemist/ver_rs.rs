// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::process::exit;

use alchemist::simpleversion::{VersionComponent, VersionComponents};
use anyhow::{bail, Context, Result};
use clap::{arg, command, Parser};

use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::combinator::eof;
use nom::sequence::preceded;
use nom::{
    combinator::{map_res, opt},
    sequence::tuple,
    IResult,
};

#[derive(Parser, Debug, PartialEq, Eq)]
#[command(name = "ver_rs")]
#[command(author = "ChromiumOS Authors")]
#[command(about = "Compares package versions", long_about = None)]
pub struct Args {
    // We need to use a Vec because parsing pairs and an optional version
    // parameter is tricky.
    #[arg(allow_hyphen_values = true)]
    args: Vec<String>,
}

fn processes(args: Args) -> Result<String> {
    let mut args = args.args;
    if args.len() < 2 {
        bail!("Usage: ver_rs <range> <repl> [<range> <repl>...] [<version>]");
    }

    let version: String = if args.len() % 2 == 0 {
        std::env::var("PV").context("PV environment variable is not set")?
    } else {
        args.pop().unwrap() // Checked size above
    };

    let mut components: VersionComponents = version.parse()?;
    if let Some(component) = components.components.first() {
        let start_idx = match component {
            VersionComponent::Component(_) => 1,
            VersionComponent::Separator(_) => 0,
        };

        for chunks in args.chunks_exact(2) {
            let range = &chunks[0];
            let (start, end) =
                parse_range(range).with_context(|| format!("Failed to parse '{}'", range))?;

            let replacement = &chunks[1];

            let mut idx = start_idx;

            for component in &mut components.components {
                if let VersionComponent::Separator(_) = component {
                    if idx >= start && end.map_or(true, |end| idx <= end) {
                        *component = VersionComponent::Separator(replacement.clone());
                    }

                    idx += 1;
                }
            }
        }
    }

    Ok(format!("{}", components))
}

// We don't use Range or RangeFrom because they are non-object safe :/
fn range_expression(input: &str) -> IResult<&str, (u32, Option<u32>)> {
    let (input, (start, range)) = tuple((
        map_res(digit1, str::parse::<u32>),
        opt(preceded(tag("-"), opt(map_res(digit1, str::parse::<u32>)))),
    ))(input)?;

    let (input, _) = eof(input)?;

    let end = match range {
        Some(end) => end,
        None => Some(start),
    };

    Ok((input, (start, end)))
}

fn parse_range(input: &str) -> Result<(u32, Option<u32>)> {
    let (_, result) = range_expression(input).map_err(|err| err.to_owned())?;
    Ok(result)
}

pub fn ver_rs_main(args: Args) -> Result<()> {
    match processes(args) {
        Ok(result) => {
            println!("{}", result);
            exit(0)
        }
        Err(e) => {
            eprintln!("{}", e);
            exit(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table() -> Result<()> {
        /*
        Taken from https://mgorny.pl/articles/the-ultimate-guide-to-eapi-7.html#replacing-version-separators-ver-rs

        Range   1.2.3   2 Ab 9 s    A.4.    .11.2.
        0       1.2.3   2 Ab 9 s    A.4.    #11.2.
        0-1     1#2.3   2#Ab 9 s    A#4.    #11#2.
        1       1#2.3   2#Ab 9 s    A#4.    .11#2.
        1-      1#2#3   2#Ab#9#s    A#4#    .11#2#
        1-2     1#2#3   2#Ab#9 s    A#4#    .11#2#
        2       1.2#3   2 Ab#9 s    A.4#    .11.2#
        2-3     1.2#3   2 Ab#9#s    A.4#    .11.2#
        3       1.2.3   2 Ab 9#s    A.4.    .11.2.
        */
        let inputs = ["1.2.3", "2 Ab 9 s", "A.4.", ".11.2."];
        let table = [
            ("0", ["1.2.3", "2 Ab 9 s", "A.4.", "#11.2."]),
            ("0-1", ["1#2.3", "2#Ab 9 s", "A#4.", "#11#2."]),
            ("1", ["1#2.3", "2#Ab 9 s", "A#4.", ".11#2."]),
            ("1-", ["1#2#3", "2#Ab#9#s", "A#4#", ".11#2#"]),
            ("1-2", ["1#2#3", "2#Ab#9 s", "A#4#", ".11#2#"]),
            ("2", ["1.2#3", "2 Ab#9 s", "A.4#", ".11.2#"]),
            ("2-3", ["1.2#3", "2 Ab#9#s", "A.4#", ".11.2#"]),
            ("3", ["1.2.3", "2 Ab 9#s", "A.4.", ".11.2."]),
        ];

        for (range, expected_values) in table {
            for (i, input) in inputs.iter().enumerate() {
                let args = Args {
                    args: vec![range.to_string(), "#".to_string(), input.to_string()],
                };

                let result = processes(args)?;

                assert_eq!(expected_values[i], result,);
            }
        }

        Ok(())
    }

    #[test]
    fn test_multiple() -> Result<()> {
        let args = Args {
            args: vec![
                "2-3".to_string(),
                "#".to_string(),
                "3-4".to_string(),
                "-".to_string(),
                "1.2.3.4.5.6".to_string(),
            ],
        };

        let result = processes(args)?;

        assert_eq!("1.2#3-4-5.6", result,);

        Ok(())
    }

    #[test]
    fn test_empty() -> Result<()> {
        let args = Args {
            args: vec!["2-3".to_string(), "#".to_string(), "".to_string()],
        };

        let result = processes(args)?;

        assert_eq!("", result,);

        Ok(())
    }
}
