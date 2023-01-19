// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{cmp::Ordering, process::exit};

use anyhow::{anyhow, bail, Result};
use clap::{arg, command, Parser};
use itertools::Itertools;
use version::Version;

#[derive(Parser, Debug, PartialEq, Eq)]
#[command(name = "ver_test")]
#[command(author = "ChromiumOS Authors")]
#[command(about = "Compares package versions", long_about = None)]
pub struct Args {
    // We need to use a Vec because having an optional first parameter
    // is not supported when using allow_hyphen_values.
    // See https://github.com/clap-rs/clap/issues/4649
    #[arg(allow_hyphen_values = true)]
    args: Vec<String>,
}

fn compare(args: Args) -> Result<bool> {
    let mut args = args.args;
    if args.len() == 2 {
        args.insert(0, std::env::var("PVR").unwrap_or_default());
    }
    let (lhs, op, rhs) = args
        .into_iter()
        .collect_tuple()
        .ok_or_else(|| anyhow!("Needs 2 or 3 arguments"))?;

    let lhs = Version::try_new(&lhs)?;
    let rhs = Version::try_new(&rhs)?;
    let ord = lhs.cmp(&rhs);

    let ok = match op.as_str() {
        "-eq" => ord == Ordering::Equal,
        "-ne" => ord != Ordering::Equal,
        "-gt" => ord == Ordering::Greater,
        "-ge" => ord != Ordering::Less,
        "-lt" => ord == Ordering::Less,
        "-le" => ord != Ordering::Greater,
        _ => bail!("Unsupported operator: {}", &op),
    };

    Ok(ok)
}

pub fn ver_test_main(args: Args) -> Result<()> {
    exit(if compare(args)? { 0 } else { 1 });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_three_args() -> Result<()> {
        let args = Args::try_parse_from(vec!["ver_test", "0.3", "-gt", "0.2"])?;
        assert_eq!(
            args,
            Args {
                args: vec!["0.3".to_owned(), "-gt".to_owned(), "0.2".to_owned()]
            }
        );

        Ok(())
    }

    #[test]
    fn test_two_args() -> Result<()> {
        let args = Args::try_parse_from(vec!["ver_test", "-gt", "0.2"])?;
        assert_eq!(
            args,
            Args {
                args: vec!["-gt".to_owned(), "0.2".to_owned()]
            }
        );

        Ok(())
    }

    #[test]
    fn test_gt() -> Result<()> {
        assert_eq!(
            true,
            compare(Args {
                args: vec!["0.5".to_owned(), "-gt".to_owned(), "0.2".to_owned()]
            })?
        );

        assert_eq!(
            false,
            compare(Args {
                args: vec!["0.2".to_owned(), "-gt".to_owned(), "0.5".to_owned()]
            })?
        );

        Ok(())
    }
}
