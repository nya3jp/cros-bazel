// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use binarypackage::BinaryPackage;
use bzip2::read::BzDecoder;
use clap::Parser;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
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
    /// Portage binary packages to validate
    #[arg(long)]
    package: PathBuf,

    /// Comma separated list of USE flags the package should have been built with.
    #[arg(
        long,
        value_parser = use_flag_arg_parser,
    )]
    use_flags: Option<HashMap<String, bool>>,

    /// Bazel requires a file to be generated for all actions
    #[arg(long, hide = true)]
    touch: Option<PathBuf>,

    /// Checks if the environment has non-hermetic variables declared.
    /// e.g., MAKEOPTS, SRANDOM, etc.
    #[arg(long)]
    check_non_hermetic_variables: bool,

    /// Do not exit abnormally on finding package differences
    #[arg(long)]
    report_only: bool,
}

/// Returns a set of the USE flags that were enabled when the package was built.
/// This doesn't include any flags that were disabled.
fn extract_use_flags(package: &binarypackage::BinaryPackage) -> Result<HashSet<&str>> {
    let use_flags = std::str::from_utf8(
        package
            .xpak()
            .get("USE")
            .context("USE XPAK entry not found")?,
    )?;

    Ok(use_flags.split_whitespace().collect())
}

/// Returns a set of all possible USE flags that could have been enabled
/// when the package was built.
fn extract_iuse_effective_flags(package: &binarypackage::BinaryPackage) -> Result<HashSet<&str>> {
    let use_flags = std::str::from_utf8(
        package
            .xpak()
            .get("IUSE_EFFECTIVE")
            .context("IUSE_EFFECTIVE XPAK entry not found")?,
    )?;

    Ok(use_flags.split_whitespace().collect())
}

fn extract_environment(pkg: &BinaryPackage) -> Result<String> {
    let bz2_env = pkg
        .xpak()
        .get("environment.bz2")
        .context("Failed to locate environment.bz2")?;

    let mut decoder = BzDecoder::new(bz2_env.as_slice());

    let mut env = String::new();
    decoder
        .read_to_string(&mut env)
        .context("Failed to decompress environment.bz2")?;

    Ok(env)
}

/// Inspects the environment for a `declare VARIABLE` statement and returns it.
///
/// This function only tries to be best effort. It doesn't try and parse
/// bash functions.
fn contains_declare<'a>(env: &'a str, var: &str) -> Option<&'a str> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"(?m)^declare(:?\s+-[\w\-]+)?\s+([\w_]+)(?:=.*$|$)").unwrap();
    }

    RE.captures_iter(env)
        .find(|c| &c[2] == var)
        .map(|c| c.get(0).unwrap().as_str())
}

fn validate_env(package: &BinaryPackage) -> Result<()> {
    let env = extract_environment(package)?;

    let errors = [
        "EPOCHREALTIME",
        "EPOCHSECONDS",
        "GCE_METADATA_HOST",
        "MAKEOPTS",
        "NINJAOPTS",
        "REPROXY_CFG_FILE",
        "SRANDOM",
        "USE_REMOTEEXEC",
    ]
    .into_iter()
    .filter_map(|var| {
        contains_declare(&env, var).map(|declare_line| {
            format!(
                "Found non hermetic variable {} in environment.bz2: {}",
                var, declare_line
            )
        })
    })
    .collect_vec();

    if errors.is_empty() {
        Ok(())
    } else {
        bail!(errors.join("\n"))
    }
}

pub fn do_validate_package(args: ValidatePackageArgs) -> Result<()> {
    let package =
        BinaryPackage::open(&args.package).with_context(|| format!("{:?}", args.package))?;

    // Reports a validation error. If --report-only is set, it just prints the error to stdout and
    // return success. Otherwise, it returns an error value with the given message.
    let report = |message: std::fmt::Arguments| -> Result<()> {
        if args.report_only {
            println!("{}", message);
            return Ok(());
        }
        bail!("{}", message)
    };

    if let Some(input_use_flags) = args.use_flags {
        let actual_use_flags = extract_use_flags(&package)?;
        let expected_use_flags = input_use_flags
            .iter()
            .filter_map(|(k, v)| v.then_some(k.as_ref()))
            .collect();

        if actual_use_flags != expected_use_flags {
            report(format_args!(
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
            ))?;
        }

        let actual_iuse_effective_flags = extract_iuse_effective_flags(&package)?;
        let expected_iuse_effective_flags = input_use_flags.keys().map(|k| k.as_ref()).collect();

        if actual_iuse_effective_flags != expected_iuse_effective_flags {
            report(format_args!(
                "\n* IUSE Flag mismatch!\n  \
                Expected IUSE: {}\n  \
                Actual IUSE: {}\n  \
                Extra flags: {}\n  \
                Missing flags: {}\n  \
                ",
                expected_iuse_effective_flags.iter().sorted().join(", "),
                actual_iuse_effective_flags.iter().sorted().join(", "),
                actual_iuse_effective_flags
                    .difference(&expected_iuse_effective_flags)
                    .sorted()
                    .join(", "),
                expected_iuse_effective_flags
                    .difference(&actual_iuse_effective_flags)
                    .sorted()
                    .join(", "),
            ))?;
        }
    }

    if args.check_non_hermetic_variables {
        if let Err(env_error) = validate_env(&package) {
            report(format_args!("{}", env_error))?;
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
                report_only: false,
                check_non_hermetic_variables: false,
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
                report_only: false,
                check_non_hermetic_variables: false,
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
                report_only: false,
                check_non_hermetic_variables: false,
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
                "abi_x86_64",
                "amd64",
                "ncurses",
                "spell",
                "kernel_linux",
                "userland_GNU",
                "elibc_glibc",
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
                ("abi_x86_64".into(), true),
                ("alpha".into(), false),
                ("amd64".into(), true),
                ("amd64-fbsd".into(), false),
                ("amd64-linux".into(), false),
                ("arm".into(), false),
                ("arm-linux".into(), false),
                ("arm64".into(), false),
                ("debug".into(), false),
                ("elibc_AIX".into(), false),
                ("elibc_bionic".into(), false),
                ("elibc_Cygwin".into(), false),
                ("elibc_Darwin".into(), false),
                ("elibc_DragonFly".into(), false),
                ("elibc_FreeBSD".into(), false),
                ("elibc_glibc".into(), false),
                ("elibc_glibc".into(), true),
                ("elibc_HPUX".into(), false),
                ("elibc_Interix".into(), false),
                ("elibc_mingw".into(), false),
                ("elibc_mintlib".into(), false),
                ("elibc_musl".into(), false),
                ("elibc_NetBSD".into(), false),
                ("elibc_OpenBSD".into(), false),
                ("elibc_SunOS".into(), false),
                ("elibc_uclibc".into(), false),
                ("elibc_Winnt".into(), false),
                ("hppa".into(), false),
                ("hppa-hpux".into(), false),
                ("ia64".into(), false),
                ("ia64-hpux".into(), false),
                ("ia64-linux".into(), false),
                ("justify".into(), false),
                ("kernel_AIX".into(), false),
                ("kernel_Darwin".into(), false),
                ("kernel_FreeBSD".into(), false),
                ("kernel_freemint".into(), false),
                ("kernel_HPUX".into(), false),
                ("kernel_linux".into(), false),
                ("kernel_linux".into(), true),
                ("kernel_NetBSD".into(), false),
                ("kernel_OpenBSD".into(), false),
                ("kernel_SunOS".into(), false),
                ("kernel_Winnt".into(), false),
                ("m68k".into(), false),
                ("m68k-mint".into(), false),
                ("magic".into(), false),
                ("minimal".into(), false),
                ("mips".into(), false),
                ("ncurses".into(), true),
                ("nios2".into(), false),
                ("nls".into(), false),
                ("ppc".into(), false),
                ("ppc-aix".into(), false),
                ("ppc-macos".into(), false),
                ("ppc-openbsd".into(), false),
                ("ppc64".into(), false),
                ("ppc64-linux".into(), false),
                ("prefix".into(), false),
                ("prefix-guest".into(), false),
                ("prefix-stack".into(), false),
                ("riscv".into(), false),
                ("s390".into(), false),
                ("sh".into(), false),
                ("sparc".into(), false),
                ("sparc-fbsd".into(), false),
                ("sparc-solaris".into(), false),
                ("sparc64-freebsd".into(), false),
                ("sparc64-solaris".into(), false),
                ("spell".into(), true),
                ("split-usr".into(), false),
                ("static".into(), false),
                ("unicode".into(), false),
                ("userland_BSD".into(), false),
                ("userland_GNU".into(), true),
                ("x64-cygwin".into(), false),
                ("x64-freebsd".into(), false),
                ("x64-macos".into(), false),
                ("x64-openbsd".into(), false),
                ("x64-solaris".into(), false),
                ("x86".into(), false),
                ("x86-cygwin".into(), false),
                ("x86-fbsd".into(), false),
                ("x86-freebsd".into(), false),
                ("x86-interix".into(), false),
                ("x86-linux".into(), false),
                ("x86-macos".into(), false),
                ("x86-netbsd".into(), false),
                ("x86-openbsd".into(), false),
                ("x86-solaris".into(), false),
                ("x86-winnt".into(), false),
            ])),
            report_only: false,
            check_non_hermetic_variables: false,
        };

        do_validate_package(args)?;

        Ok(())
    }

    #[test]
    fn use_flags_not_equal() -> Result<()> {
        let args = ValidatePackageArgs {
            package: testdata(BINPKG)?,
            touch: None,
            use_flags: Some(HashMap::from([("foo".into(), true), ("bar".into(), false)])),
            report_only: false,
            check_non_hermetic_variables: false,
        };

        assert!(do_validate_package(args).is_err());

        Ok(())
    }

    #[test]
    fn use_flags_not_equal_report_only() -> Result<()> {
        let args = ValidatePackageArgs {
            package: testdata(BINPKG)?,
            touch: None,
            use_flags: Some(HashMap::from([("foo".into(), true), ("bar".into(), false)])),
            report_only: true,
            check_non_hermetic_variables: true,
        };

        do_validate_package(args)?;

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
            report_only: false,
            check_non_hermetic_variables: false,
        };

        do_validate_package(args)?;

        assert!(validation.exists());

        Ok(())
    }

    #[test]
    fn declare_match() -> Result<()> {
        let matches = [
            ("declare -- foo", true),
            ("declare -- foo=\"\"", true),
            ("declare -x foo=\"\"", true),
            ("declare -i foo=\"0\"", true),
            ("declare foo", true),
            ("declare foo=\"bar\"", true),
            ("foo()", false),
            // Assume we are in a function.
            ("  declare -x foo", false),
        ];

        for (env, expected) in matches {
            let actual = contains_declare(env, "foo");
            assert_eq!(
                actual,
                if expected { Some(env) } else { None },
                "env: '{}'",
                env
            );
        }

        assert_eq!(
            contains_declare(
                r#"declare baz
declare foo="Hello"
declare bar
"#,
                "foo"
            ),
            Some("declare foo=\"Hello\"")
        );

        Ok(())
    }

    #[test]
    fn package_conatins_invalid_environment() -> Result<()> {
        let package = BinaryPackage::open(&testdata(BINPKG)?)?;

        match validate_env(&package) {
            Ok(_) => bail!("Package should fail validation!"),
            Err(e) => assert!(
                e.to_string().starts_with(
r#"Found non hermetic variable EPOCHREALTIME in environment.bz2: declare -- EPOCHREALTIME="1665539747.245142"
Found non hermetic variable EPOCHSECONDS in environment.bz2: declare -- EPOCHSECONDS="1665539747"
Found non hermetic variable MAKEOPTS in environment.bz2: declare -x MAKEOPTS="-j96"
Found non hermetic variable SRANDOM in environment.bz2: declare -i SRANDOM="1404730306""#),
                "{}",
                e
            ),
        };

        Ok(())
    }

    #[test]
    fn package_conatins_valid_environment() -> Result<()> {
        let package = BinaryPackage::open(&testdata(BINPKG_CLEAN_ENV)?)?;

        validate_env(&package)?;

        Ok(())
    }
}
