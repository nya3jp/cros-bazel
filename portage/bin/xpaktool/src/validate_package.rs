// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use binarypackage::BinaryPackage;
use clap::Parser;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::PathBuf;

fn use_flag_arg_parser(
    value: &str,
) -> std::result::Result<HashMap<String, bool>, clap::error::Error> {
    if value.is_empty() {
        return Ok(HashMap::new());
    }
    Ok(value
        .split(',')
        .map(|flag| {
            if let Some(flag) = flag.strip_prefix('+') {
                (flag.to_string(), true)
            } else if let Some(flag) = flag.strip_prefix('-') {
                (flag.to_string(), false)
            } else {
                (flag.to_string(), true)
            }
        })
        .collect())
}

/// Performs validations on a binpkg.
#[derive(Parser, Debug, PartialEq, Eq)]
pub struct ValidatePackageArgs {
    #[arg(long, help = "Portage binary packages to validate")]
    package: PathBuf,

    #[arg(
        long,
        help = "Comma separated list of USE flags the package should have been built with.",
        value_parser = use_flag_arg_parser,
    )]
    use_flags: Option<HashMap<String, bool>>,

    #[arg(
        long,
        help = "Bazel requires a file to be generated for all actions",
        hide = true
    )]
    touch: Option<PathBuf>,
}

fn extract_use_flags(package: &BinaryPackage) -> Result<HashSet<String>> {
    let use_flags = std::str::from_utf8(
        package
            .xpak()
            .get("USE")
            .context("USE XPAK entry not found")?,
    )?;

    Ok(use_flags
        .split_whitespace()
        .map(|flag| flag.to_string())
        .collect())
}

pub fn do_validate_package(args: ValidatePackageArgs) -> Result<()> {
    let package =
        BinaryPackage::open(&args.package).with_context(|| format!("{:?}", args.package))?;

    if let Some(expected_use_flags) = args.use_flags {
        let actual_use_flags = extract_use_flags(&package)?;
        let expected_use_flags = expected_use_flags
            .into_iter()
            .filter_map(|(k, v)| v.then_some(k))
            .collect();

        if actual_use_flags != expected_use_flags {
            bail!(
                "\n* USE Flag mismatch!\n  \
                Expected USE: {}\n  \
                Actual USE: {}\n  \
                Extra flags: {}\n  \
                Missing flags: {}\n  \
                ",
                expected_use_flags.iter().sorted().join(", "),
                actual_use_flags.iter().sorted().join(", "),
                actual_use_flags
                    .difference(&expected_use_flags)
                    .sorted()
                    .join(", "),
                expected_use_flags
                    .difference(&actual_use_flags)
                    .sorted()
                    .join(", "),
            );
        }
    }

    if let Some(touch) = args.touch {
        File::create(&touch).with_context(|| format!("touch file: {touch:?}"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdata::*;

    #[test]
    fn args_no_useflags() -> Result<()> {
        let args = ValidatePackageArgs::try_parse_from([
            "FOO",
            "--package",
            "foo.tbz2",
            "--touch",
            "touch",
        ])?;

        assert_eq!(
            args,
            ValidatePackageArgs {
                package: PathBuf::from("foo.tbz2"),
                touch: Some(PathBuf::from("touch")),
                use_flags: None,
            }
        );

        Ok(())
    }

    #[test]
    fn args_useflags() -> Result<()> {
        let args = ValidatePackageArgs::try_parse_from([
            "FOO",
            "--package",
            "foo.tbz2",
            "--use-flags",
            "+foo,-bar",
        ])?;

        assert_eq!(
            args,
            ValidatePackageArgs {
                package: PathBuf::from("foo.tbz2"),
                touch: None,
                use_flags: Some(HashMap::from([
                    ("foo".to_string(), true),
                    ("bar".to_string(), false)
                ])),
            }
        );

        Ok(())
    }

    #[test]
    fn args_empty_useflags() -> Result<()> {
        let args = ValidatePackageArgs::try_parse_from([
            "FOO",
            "--package",
            "foo.tbz2",
            "--use-flags",
            "",
        ])?;

        assert_eq!(
            args,
            ValidatePackageArgs {
                package: PathBuf::from("foo.tbz2"),
                touch: None,
                use_flags: Some(HashMap::new()),
            }
        );

        Ok(())
    }

    #[test]
    fn extract_use_flags() -> Result<()> {
        let package = BinaryPackage::open(&testdata(BINPKG)?)?;

        let flags = super::extract_use_flags(&package)?;

        assert_eq!(
            flags,
            HashSet::from([
                "abi_x86_64".to_string(),
                "amd64".to_string(),
                "ncurses".to_string(),
                "spell".to_string(),
                "kernel_linux".to_string(),
                "userland_GNU".to_string(),
                "elibc_glibc".to_string(),
            ])
        );

        Ok(())
    }

    #[test]
    fn use_flags_equal() -> Result<()> {
        let args = ValidatePackageArgs {
            package: testdata(BINPKG)?,
            touch: None,
            use_flags: Some(HashMap::from([
                ("abi_x86_64".to_string(), true),
                ("abi_arm".to_string(), false),
                ("abi_arm64".to_string(), false),
                ("amd64".to_string(), true),
                ("arm".to_string(), false),
                ("ncurses".to_string(), true),
                ("spell".to_string(), true),
                ("kernel_linux".to_string(), true),
                ("userland_GNU".to_string(), true),
                ("elibc_glibc".to_string(), true),
            ])),
        };

        do_validate_package(args)?;

        Ok(())
    }

    #[test]
    fn use_flags_not_equal() -> Result<()> {
        let args = ValidatePackageArgs {
            package: testdata(BINPKG)?,
            touch: None,
            use_flags: Some(HashMap::from([
                ("foo".to_string(), true),
                ("bar".to_string(), false),
            ])),
        };

        assert!(do_validate_package(args).is_err());

        Ok(())
    }

    #[test]
    fn touch() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        let validation = dir.join("validation");

        let args = ValidatePackageArgs {
            package: testdata(BINPKG)?,
            touch: Some(validation.clone()),
            use_flags: None,
        };

        do_validate_package(args)?;

        assert!(validation.exists());

        Ok(())
    }
}
