// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Error, Result};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::{complete::multispace0, is_alphanumeric},
    combinator::{eof, map, opt, recognize, value},
    multi::separated_list1,
    sequence::{delimited, pair},
    IResult,
};
use nom_regex::str::re_find;
use once_cell::sync::Lazy;
use regex::Regex;
use version::Version;

use crate::dependency::{
    package::{
        PackageBlock, PackageDependency, PackageDependencyAtom, PackageSlotDependency,
        PackageUseDependency, PackageVersionDependency, PackageVersionOp,
    },
    parser::{DependencyParser, DependencyParserCommon},
    CompositeDependency, Dependency,
};

use super::PackageUseDependencyOp;

/// Raw regular expression string matching a valid package name.
pub const PACKAGE_NAME_RE_RAW: &str = r"[A-Za-z0-9_][A-Za-z0-9+_.-]*/[A-Za-z0-9_][A-Za-z0-9+_-]*";

/// Regular expression matching a string starting with a valid package name.
static PACKAGE_NAME_PLAIN_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(&format!("^{}", PACKAGE_NAME_RE_RAW)).unwrap());

/// Regular expression matching a string starting with a valid package name
/// followed by a hyphen and a valid package version.
static PACKAGE_NAME_WITH_VERSION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(
        "^{}-{}",
        PACKAGE_NAME_RE_RAW,
        version::VERSION_RE_RAW
    ))
    .unwrap()
});

/// Implements the package dependency expression parser.
pub struct PackageDependencyParser {}

impl<'i> DependencyParserCommon<'i, PackageDependencyAtom> for PackageDependencyParser {
    fn new_all_of(children: Vec<PackageDependency>) -> PackageDependency {
        Dependency::new_composite(CompositeDependency::AllOf { children })
    }

    fn expression(input: &str) -> IResult<&str, PackageDependency> {
        let (input, _) = multispace0(input)?;
        alt((
            map(Self::atom, Dependency::Leaf),
            Self::all_of,
            Self::any_of,
            Self::use_conditional,
        ))(input)
    }
}

impl PackageDependencyParser {
    fn block(input: &str) -> IResult<&str, PackageBlock> {
        alt((
            value(PackageBlock::Strong, tag(PackageBlock::Strong.as_ref())),
            value(PackageBlock::Weak, tag(PackageBlock::Weak.as_ref())),
            value(PackageBlock::None, tag(PackageBlock::None.as_ref())),
        ))(input)
    }

    fn package_name_plain(input: &str) -> IResult<&str, &str> {
        re_find(PACKAGE_NAME_PLAIN_RE.clone())(input)
    }

    fn package_name_with_version(input: &str) -> IResult<&str, (&str, PackageVersionDependency)> {
        let (input, op) = alt((
            value(
                PackageVersionOp::Equal { wildcard: false },
                tag(PackageVersionOp::Equal { wildcard: false }.as_ref()),
            ),
            value(
                PackageVersionOp::EqualExceptRevision,
                tag(PackageVersionOp::EqualExceptRevision.as_ref()),
            ),
            value(
                PackageVersionOp::GreaterOrEqual,
                tag(PackageVersionOp::GreaterOrEqual.as_ref()),
            ),
            value(
                PackageVersionOp::Greater,
                tag(PackageVersionOp::Greater.as_ref()),
            ),
            value(
                PackageVersionOp::LessOrEqual,
                tag(PackageVersionOp::LessOrEqual.as_ref()),
            ),
            value(PackageVersionOp::Less, tag(PackageVersionOp::Less.as_ref())),
        ))(input)?;
        let (input, package_name_and_version) =
            re_find(PACKAGE_NAME_WITH_VERSION_RE.clone())(input)?;
        let (package_name, version) = Version::from_str_suffix(package_name_and_version).unwrap();
        let (input, op) = {
            match op {
                PackageVersionOp::Equal { .. } => {
                    let (input, mark) = opt(tag("*"))(input)?;
                    let wildcard = mark.is_some();
                    (input, PackageVersionOp::Equal { wildcard })
                }
                _ => (input, op),
            }
        };
        Ok((
            input,
            (package_name, PackageVersionDependency::new(op, version)),
        ))
    }

    fn slot_name_unit(input: &str) -> IResult<&str, &str> {
        recognize(pair(
            take_while1(|c| is_alphanumeric(c as u8) || c == '_'),
            take_while(|c| {
                is_alphanumeric(c as u8) || c == '_' || c == '+' || c == '.' || c == '-'
            }),
        ))(input)
    }

    fn slot_name(input: &str) -> IResult<&str, &str> {
        recognize(pair(
            Self::slot_name_unit,
            opt(pair(tag("/"), Self::slot_name_unit)),
        ))(input)
    }

    fn slot_specific(input: &str) -> IResult<&str, PackageSlotDependency> {
        let (input, (spec, opt_mark)) = pair(Self::slot_name, opt(tag("=")))(input)?;
        let (main, sub) = spec
            .split_once('/')
            .map(|(main, sub)| (main.to_owned(), Some(sub.to_owned())))
            .unwrap_or((spec.to_owned(), None));
        Ok((
            input,
            PackageSlotDependency::new(Some((main, sub)), opt_mark.is_some()),
        ))
    }

    fn slot_wildcard(input: &str) -> IResult<&str, PackageSlotDependency> {
        let (input, mark) = alt((tag("*"), tag("=")))(input)?;
        Ok((input, PackageSlotDependency::new(None, mark == "=")))
    }

    fn slot(input: &str) -> IResult<&str, PackageSlotDependency> {
        let (input, _) = tag(":")(input)?;
        alt((Self::slot_specific, Self::slot_wildcard))(input)
    }

    fn use_item_default(input: &str) -> IResult<&str, bool> {
        delimited(
            tag("("),
            alt((value(true, tag("+")), value(false, tag("-")))),
            tag(")"),
        )(input)
    }

    fn use_item(input: &str) -> IResult<&str, PackageUseDependency> {
        let (input, negate) = opt(tag("-"))(input)?;
        if negate.is_some() {
            let (input, (flag, missing_default)) =
                pair(Self::use_name, opt(Self::use_item_default))(input)?;

            return Ok((
                input,
                PackageUseDependency {
                    negate: true,
                    flag: flag.to_string(),
                    op: PackageUseDependencyOp::Required,
                    missing_default,
                },
            ));
        }

        let (input, not_op) = opt(tag("!"))(input)?;
        if not_op.is_some() {
            let (input, (flag, missing_default)) =
                pair(Self::use_name, opt(Self::use_item_default))(input)?;

            let (input, op) = alt((
                value(PackageUseDependencyOp::Synchronized, tag("=")),
                value(PackageUseDependencyOp::ConditionalRequired, tag("?")),
            ))(input)?;

            return Ok((
                input,
                PackageUseDependency {
                    negate: true,
                    flag: flag.to_string(),
                    op,
                    missing_default,
                },
            ));
        }

        let (input, (flag, missing_default)) =
            pair(Self::use_name, opt(Self::use_item_default))(input)?;

        let (input, op) = opt(alt((
            value(PackageUseDependencyOp::Synchronized, tag("=")),
            value(PackageUseDependencyOp::ConditionalRequired, tag("?")),
        )))(input)?;

        let op = op.unwrap_or(PackageUseDependencyOp::Required);

        Ok((
            input,
            PackageUseDependency {
                negate: negate.is_some(),
                flag: flag.to_string(),
                op,
                missing_default,
            },
        ))
    }

    fn uses(input: &str) -> IResult<&str, Vec<PackageUseDependency>> {
        delimited(
            tag("["),
            separated_list1(tag(","), Self::use_item),
            tag("]"),
        )(input)
    }

    fn atom(input: &str) -> IResult<&str, PackageDependencyAtom> {
        let (input, block) = Self::block(input)?;
        let (input, (package_name, version)) = alt((
            map(Self::package_name_plain, |name| (name, None)),
            map(Self::package_name_with_version, |(name, op)| {
                (name, Some(op))
            }),
        ))(input)?;
        let (input, slot) = opt(Self::slot)(input)?;
        let (input, uses) = opt(Self::uses)(input)?;
        Ok((
            input,
            PackageDependencyAtom {
                package_name: package_name.to_owned(),
                version,
                slot,
                uses: uses.unwrap_or_default(),
                block,
            },
        ))
    }

    fn full_atom(input: &str) -> IResult<&str, PackageDependencyAtom> {
        let (input, atom) = Self::atom(input)?;
        let (input, _) = eof(input)?;
        Ok((input, atom))
    }

    fn full(input: &str) -> IResult<&str, PackageDependency> {
        let (input, children) = Self::expression_list(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = eof(input)?;
        Ok((input, Self::new_all_of(children)))
    }

    pub fn parse_atom(input: &str) -> Result<PackageDependencyAtom> {
        let (_, atom) = PackageDependencyParser::full_atom(input).map_err(|err| err.to_owned())?;
        Ok(atom)
    }
}

impl DependencyParser<PackageDependency> for PackageDependencyParser {
    type Err = Error;

    fn parse(input: &str) -> Result<PackageDependency> {
        let (_, deps) = Self::full(input).map_err(|err| err.to_owned())?;
        Ok(deps)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, str::FromStr};

    use anyhow::Result;
    use nom::sequence::terminated;

    use crate::dependency::package::PackageUseDependencyOp;

    use super::*;

    #[test]
    fn test_parse_simple() -> Result<()> {
        let expr = PackageDependencyParser::parse_atom("sys-apps/systemd-utils")?;

        assert_eq!(
            expr,
            PackageDependencyAtom {
                package_name: "sys-apps/systemd-utils".to_owned(),
                version: None,
                slot: None,
                uses: vec![],
                block: PackageBlock::None
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_version_eq() -> Result<()> {
        let expr = PackageDependencyParser::parse_atom("=sys-apps/systemd-utils-9999")?;

        assert_eq!(
            expr,
            PackageDependencyAtom {
                package_name: "sys-apps/systemd-utils".to_owned(),
                version: Some(PackageVersionDependency {
                    op: PackageVersionOp::Equal { wildcard: false },
                    version: Version::from_str("9999")?,
                }),
                slot: None,
                uses: vec![],
                block: PackageBlock::None
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_version_eq_wildcard() -> Result<()> {
        let expr = PackageDependencyParser::parse_atom("=sys-apps/systemd-utils-9*")?;

        assert_eq!(
            expr,
            PackageDependencyAtom {
                package_name: "sys-apps/systemd-utils".to_owned(),
                version: Some(PackageVersionDependency {
                    op: PackageVersionOp::Equal { wildcard: true },
                    version: Version::from_str("9")?,
                }),
                slot: None,
                uses: vec![],
                block: PackageBlock::None
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_slot() -> Result<()> {
        let expr = PackageDependencyParser::parse_atom("sys-apps/systemd-utils:1")?;

        assert_eq!(
            expr,
            PackageDependencyAtom {
                package_name: "sys-apps/systemd-utils".to_owned(),
                version: None,
                slot: Some(PackageSlotDependency {
                    slot: Some(("1".to_owned(), None)),
                    rebuild_on_slot_change: false,
                }),
                uses: vec![],
                block: PackageBlock::None
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_slot_rebuild() -> Result<()> {
        let expr = PackageDependencyParser::parse_atom("sys-apps/systemd-utils:1=")?;

        assert_eq!(
            expr,
            PackageDependencyAtom {
                package_name: "sys-apps/systemd-utils".to_owned(),
                version: None,
                slot: Some(PackageSlotDependency {
                    slot: Some(("1".to_owned(), None)),
                    rebuild_on_slot_change: true,
                }),
                uses: vec![],
                block: PackageBlock::None
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_use_item() -> Result<()> {
        let test_cases = HashMap::from([
            (
                "udev",
                PackageUseDependency {
                    negate: false,
                    flag: "udev".to_owned(),
                    op: PackageUseDependencyOp::Required,
                    missing_default: None,
                },
            ),
            (
                "-udev",
                PackageUseDependency {
                    negate: true,
                    flag: "udev".to_owned(),
                    op: PackageUseDependencyOp::Required,
                    missing_default: None,
                },
            ),
            (
                "udev=",
                PackageUseDependency {
                    negate: false,
                    flag: "udev".to_owned(),
                    op: PackageUseDependencyOp::Synchronized,
                    missing_default: None,
                },
            ),
            (
                "!udev=",
                PackageUseDependency {
                    negate: true,
                    flag: "udev".to_owned(),
                    op: PackageUseDependencyOp::Synchronized,
                    missing_default: None,
                },
            ),
            (
                "udev?",
                PackageUseDependency {
                    negate: false,
                    flag: "udev".to_owned(),
                    op: PackageUseDependencyOp::ConditionalRequired,
                    missing_default: None,
                },
            ),
            (
                "!udev?",
                PackageUseDependency {
                    negate: true,
                    flag: "udev".to_owned(),
                    op: PackageUseDependencyOp::ConditionalRequired,
                    missing_default: None,
                },
            ),
            (
                "udev(+)",
                PackageUseDependency {
                    negate: false,
                    flag: "udev".to_owned(),
                    op: PackageUseDependencyOp::Required,
                    missing_default: Some(true),
                },
            ),
            (
                "-udev(+)",
                PackageUseDependency {
                    negate: true,
                    flag: "udev".to_owned(),
                    op: PackageUseDependencyOp::Required,
                    missing_default: Some(true),
                },
            ),
            (
                "udev(+)=",
                PackageUseDependency {
                    negate: false,
                    flag: "udev".to_owned(),
                    op: PackageUseDependencyOp::Synchronized,
                    missing_default: Some(true),
                },
            ),
            (
                "!udev(+)=",
                PackageUseDependency {
                    negate: true,
                    flag: "udev".to_owned(),
                    op: PackageUseDependencyOp::Synchronized,
                    missing_default: Some(true),
                },
            ),
            (
                "udev(+)?",
                PackageUseDependency {
                    negate: false,
                    flag: "udev".to_owned(),
                    op: PackageUseDependencyOp::ConditionalRequired,
                    missing_default: Some(true),
                },
            ),
            (
                "!udev(+)?",
                PackageUseDependency {
                    negate: true,
                    flag: "udev".to_owned(),
                    op: PackageUseDependencyOp::ConditionalRequired,
                    missing_default: Some(true),
                },
            ),
            (
                "udev(-)",
                PackageUseDependency {
                    negate: false,
                    flag: "udev".to_owned(),
                    op: PackageUseDependencyOp::Required,
                    missing_default: Some(false),
                },
            ),
            (
                "-udev(-)",
                PackageUseDependency {
                    negate: true,
                    flag: "udev".to_owned(),
                    op: PackageUseDependencyOp::Required,
                    missing_default: Some(false),
                },
            ),
            (
                "udev(-)=",
                PackageUseDependency {
                    negate: false,
                    flag: "udev".to_owned(),
                    op: PackageUseDependencyOp::Synchronized,
                    missing_default: Some(false),
                },
            ),
            (
                "!udev(-)=",
                PackageUseDependency {
                    negate: true,
                    flag: "udev".to_owned(),
                    op: PackageUseDependencyOp::Synchronized,
                    missing_default: Some(false),
                },
            ),
            (
                "udev(-)?",
                PackageUseDependency {
                    negate: false,
                    flag: "udev".to_owned(),
                    op: PackageUseDependencyOp::ConditionalRequired,
                    missing_default: Some(false),
                },
            ),
            (
                "!udev(-)?",
                PackageUseDependency {
                    negate: true,
                    flag: "udev".to_owned(),
                    op: PackageUseDependencyOp::ConditionalRequired,
                    missing_default: Some(false),
                },
            ),
        ]);

        for (input, expected) in test_cases {
            let (output, actual) = PackageDependencyParser::use_item(input)?;
            assert_eq!(0, output.len());

            assert_eq!(expected, actual, "input: {}", input);
            assert_eq!(input, format!("{}", actual));
        }

        Ok(())
    }

    #[test]
    fn test_parse_use_item_invalid() -> Result<()> {
        let test_cases = vec!["!udev", "-udev=", "-udev?"];

        for input in test_cases {
            let result = terminated(PackageDependencyParser::use_item, eof)(input);
            assert!(result.is_err(), "input: {}", input);
        }

        Ok(())
    }

    #[test]
    fn test_parse_uses() -> Result<()> {
        let expr = PackageDependencyParser::parse_atom(
            "sys-apps/systemd-utils[-udev,!selinux(-)=,boot(-)=,sysusers(+)?]",
        )?;

        assert_eq!(
            expr,
            PackageDependencyAtom {
                package_name: "sys-apps/systemd-utils".to_owned(),
                version: None,
                slot: None,
                uses: vec![
                    PackageUseDependency {
                        negate: true,
                        flag: "udev".to_owned(),
                        op: PackageUseDependencyOp::Required,
                        missing_default: None,
                    },
                    PackageUseDependency {
                        negate: true,
                        flag: "selinux".to_owned(),
                        op: PackageUseDependencyOp::Synchronized,
                        missing_default: Some(false),
                    },
                    PackageUseDependency {
                        negate: false,
                        flag: "boot".to_owned(),
                        op: PackageUseDependencyOp::Synchronized,
                        missing_default: Some(false),
                    },
                    PackageUseDependency {
                        negate: false,
                        flag: "sysusers".to_owned(),
                        op: PackageUseDependencyOp::ConditionalRequired,
                        missing_default: Some(true),
                    },
                ],
                block: PackageBlock::None
            }
        );

        Ok(())
    }
}
