// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

pub mod parser;

use anyhow::{bail, ensure, Error, Result};
use itertools::Itertools;
use std::{fmt::Display, str::FromStr};
use version::Version;

use crate::{
    config::ProvidedPackage,
    data::{Slot, UseMap},
};

use super::{parser::DependencyParserType, Dependency, Predicate};
use parser::PackageDependencyParser;

/// Alias of Dependency specialized to package dependencies.
pub type PackageDependency = Dependency<PackageDependencyAtom>;

/// A borrowed subset of package data to be passed to package-related predicates.
#[derive(Clone, Copy, Debug)]
pub struct PackageRef<'a> {
    pub package_name: &'a str,
    pub version: &'a Version,
    pub slot: Slot<&'a str>,
    pub use_map: &'a UseMap,
}

impl AsRef<UseMap> for PackageRef<'_> {
    fn as_ref(&self) -> &UseMap {
        self.use_map
    }
}

/// Similar to [`PackageRef`], but it contains an even smaller subset of fields
/// that are available before computing package USE flags.
///
/// We use this struct to work with package dependency atoms evaluated before
/// computing package USE flags, e.g. on processing `package.use`.
#[derive(Clone, Copy, Debug)]
pub struct ThinPackageRef<'a> {
    pub package_name: &'a str,
    pub version: &'a Version,
    pub slot: Slot<&'a str>,
}

/// Represents a package SLOT dependency.
///
/// This is a subcomponent of [`PackageAtomDependency`].
///
/// See the PMS for the specification:
/// https://projects.gentoo.org/pms/8/pms.html#x1-820008.3.3
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PackageSlotDependency {
    slot: Option<(String, Option<String>)>,
    rebuild_on_slot_change: bool,
}

impl PackageSlotDependency {
    pub(super) fn new(
        slot: Option<(String, Option<String>)>,
        rebuild_on_slot_change: bool,
    ) -> Self {
        Self {
            slot,
            rebuild_on_slot_change,
        }
    }
}

impl Predicate<Slot<&'_ str>> for PackageSlotDependency {
    fn matches(&self, slot: &Slot<&'_ str>) -> bool {
        match &self.slot {
            None => true,
            Some((main, sub)) => {
                if slot.main != main {
                    return false;
                }
                if let Some(sub) = sub {
                    if slot.sub != sub {
                        return false;
                    }
                }
                true
            }
        }
    }
}

impl Display for PackageSlotDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.slot {
            Some((main, sub)) => {
                write!(f, "{}", main)?;
                if let Some(sub) = sub {
                    write!(f, "/{}", sub)?;
                }
                if self.rebuild_on_slot_change {
                    write!(f, "=")?;
                }
            }
            None => {
                if self.rebuild_on_slot_change {
                    write!(f, "=")?;
                } else {
                    write!(f, "*")?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum PackageUseDependencyOp {
    /// The USE flag must be set.
    Required,
    /// The target package's USE flag must have the same value as the package
    /// declaring this dependency.
    Synchronized,
    /// The target package's USE flag must be enabled if the package declaring
    /// this dependency has the flag enabled.
    ConditionalRequired,
}

/// Represents a package USE dependency.
///
/// This is a subcomponent of [`PackageAtomDependency`].
///
/// See the PMS for the specification:
/// https://projects.gentoo.org/pms/8/pms.html#x1-830008.3.4
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackageUseDependency {
    negate: bool,
    flag: String,
    op: PackageUseDependencyOp,
    /// If the target package doesn't declare the USE flag, use the following
    /// value in the computation. If this is None and the package doesn't
    /// declare the USE flag, then an error should be reported.
    missing_default: Option<bool>,
}

impl Predicate<UseMap> for PackageUseDependency {
    fn matches(&self, _uses: &UseMap) -> bool {
        // TODO: Implement USE dependencies.
        true
    }
}

impl Display for PackageUseDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.negate {
            match self.op {
                PackageUseDependencyOp::Required => write!(f, "-")?,
                _ => write!(f, "!")?,
            }
        }

        write!(f, "{}", &self.flag)?;

        if let Some(default) = self.missing_default {
            if default {
                write!(f, "(+)")?;
            } else {
                write!(f, "(-)")?;
            }
        }

        match self.op {
            PackageUseDependencyOp::Synchronized => write!(f, "=")?,
            PackageUseDependencyOp::ConditionalRequired => write!(f, "?")?,
            _ => {}
        }

        Ok(())
    }
}

/// Enum for package version comparison operators.
///
/// See the PMS for the specification:
/// https://projects.gentoo.org/pms/8/pms.html#x1-800008.3.1
#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    strum_macros::AsRefStr,
    strum_macros::Display,
    strum_macros::EnumIter,
    strum_macros::EnumString,
)]
pub enum PackageVersionOp {
    #[strum(serialize = "<")]
    Less,
    #[strum(serialize = "<=")]
    LessOrEqual,
    #[strum(serialize = "=")]
    Equal { wildcard: bool },
    #[strum(serialize = "~")]
    EqualExceptRevision,
    #[strum(serialize = ">")]
    Greater,
    #[strum(serialize = ">=")]
    GreaterOrEqual,
}

/// Represents a package version dependency.
///
/// This is a subcomponent of [`PackageAtomDependency`].
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PackageVersionDependency {
    op: PackageVersionOp,
    version: Version,
}

impl PackageVersionDependency {
    pub(super) fn new(op: PackageVersionOp, version: Version) -> Self {
        Self { op, version }
    }
}

impl Predicate<Version> for PackageVersionDependency {
    fn matches(&self, version: &Version) -> bool {
        match self.op {
            PackageVersionOp::Equal { wildcard } => {
                if wildcard {
                    // TODO(b/272798056): Support real wildcards.
                    version.starts_with(&self.version)
                } else {
                    version == &self.version
                }
            }
            PackageVersionOp::EqualExceptRevision => version.without_revision() == self.version,
            PackageVersionOp::Less => version < &self.version,
            PackageVersionOp::LessOrEqual => version <= &self.version,
            PackageVersionOp::Greater => version > &self.version,
            PackageVersionOp::GreaterOrEqual => version >= &self.version,
        }
    }
}

/// Enum for package block operators.
///
/// See the PMS for the specification:
/// https://projects.gentoo.org/pms/8/pms.html#x1-810008.3.2
#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    strum_macros::AsRefStr,
    strum_macros::Display,
    strum_macros::EnumIter,
    strum_macros::EnumString,
)]
pub enum PackageBlock {
    #[strum(serialize = "")]
    None,
    #[strum(serialize = "!")]
    Weak,
    #[strum(serialize = "!!")]
    Strong,
}

/// Represents an atom in package dependency specifications.
///
/// This should only be used when parsing DEPEND, RDEPEND, BDEPEND, and PDEPEND.
///
/// See the PMS for the specification:
/// https://projects.gentoo.org/pms/8/pms.html#x1-790008.3
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PackageDependencyAtom {
    package_name: String,
    version: Option<PackageVersionDependency>,
    slot: Option<PackageSlotDependency>,
    uses: Vec<PackageUseDependency>,
    block: PackageBlock,
}

impl PackageDependencyAtom {
    /// Constructs a simple atom that consists of a package name only.
    pub fn new_simple(package_name: &str) -> Self {
        Self {
            package_name: package_name.to_owned(),
            version: None,
            slot: None,
            uses: Vec::new(),
            block: PackageBlock::None,
        }
    }

    pub fn package_name(&self) -> &str {
        self.package_name.as_ref()
    }
    pub fn version(&self) -> Option<&PackageVersionDependency> {
        self.version.as_ref()
    }
    pub fn slot(&self) -> Option<&PackageSlotDependency> {
        self.slot.as_ref()
    }
    pub fn uses(&self) -> &Vec<PackageUseDependency> {
        self.uses.as_ref()
    }
    pub fn block(&self) -> PackageBlock {
        self.block
    }
}

impl FromStr for PackageDependencyAtom {
    type Err = Error;

    /// Parses a package dependency atom string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PackageDependencyParser::parse_atom(s)
    }
}

impl DependencyParserType<PackageDependencyAtom> for PackageDependencyAtom {
    type Parser = PackageDependencyParser;
}

impl Display for PackageDependencyAtom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.block)?;
        if let Some(version) = &self.version {
            write!(f, "{}", version.op)?;
        }
        write!(f, "{}", &self.package_name)?;
        if let Some(version) = &self.version {
            write!(f, "-{}", version.version)?;
            if let PackageVersionOp::Equal { wildcard: true } = version.op {
                write!(f, "*")?;
            }
        }
        if let Some(slot) = &self.slot {
            write!(f, ":{}", slot)?;
        }
        if !self.uses.is_empty() {
            write!(f, "[{}]", self.uses.iter().map(|s| s.to_string()).join(","))?;
        }
        Ok(())
    }
}

impl Predicate<PackageRef<'_>> for PackageDependencyAtom {
    fn matches(&self, package: &PackageRef<'_>) -> bool {
        let match_except_block = (|| {
            if package.package_name != self.package_name {
                return false;
            }
            if let Some(p) = &self.version {
                if !p.matches(package.version) {
                    return false;
                }
            }
            if let Some(p) = &self.slot {
                if !p.matches(&package.slot) {
                    return false;
                }
            }
            if !self.uses.iter().all(|p| p.matches(package.use_map)) {
                return false;
            }
            true
        })();

        match_except_block == (self.block == PackageBlock::None)
    }
}

impl Predicate<ThinPackageRef<'_>> for PackageDependencyAtom {
    fn matches(&self, package: &ThinPackageRef<'_>) -> bool {
        let match_except_block = (|| {
            if package.package_name != self.package_name {
                return false;
            }
            if let Some(p) = &self.version {
                if !p.matches(package.version) {
                    return false;
                }
            }
            if let Some(p) = &self.slot {
                if !p.matches(&package.slot) {
                    return false;
                }
            }
            true
        })();

        match_except_block == (self.block == PackageBlock::None)
    }
}

impl Predicate<&ProvidedPackage> for PackageDependencyAtom {
    /// Checks is the [`ProvidedPackage`] matches the `atom`.
    ///
    /// A ProvidedPackage only contains a package_name and a version. This
    /// unfortunately means we can't match against `slot` or `USE` dependencies.
    /// These constraints are ignored when matching against a provided package.
    ///
    /// Due to these limitations, the EAPI7 has deprecated and strongly
    /// discourages the use of package.provided.
    fn matches(&self, package: &&ProvidedPackage) -> bool {
        let match_except_block = (|| {
            if package.package_name != self.package_name {
                return false;
            }

            if let Some(p) = &self.version {
                if !p.matches(&package.version) {
                    return false;
                }
            }
            true
        })();

        match_except_block == (self.block == PackageBlock::None)
    }
}

/// Represents a package atom.
///
/// This should be used when parsing user configuration files or user input.
///
/// See the PMS for the specification:
/// https://projects.gentoo.org/pms/8/pms.html#x1-790008.3
///
/// TODO: Do we want to implement simple USE dependencies that don't require a
/// declaring package to compute? i.e., [udev,-boot]
///
/// TODO(b/268153190): Implement repository constraints. i.e., ::portage-stable
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PackageAtom {
    package_name: String,
    version: Option<PackageVersionDependency>,
    /// Slot and Sub-Slot
    slot: Option<(String, Option<String>)>,
}

impl PackageAtom {
    pub fn package_name(&self) -> &String {
        &self.package_name
    }

    pub fn version(&self) -> &Option<PackageVersionDependency> {
        &self.version
    }

    pub fn slot(&self) -> &Option<(String, Option<String>)> {
        &self.slot
    }
}

impl FromStr for PackageAtom {
    type Err = Error;

    /// Parses a package atom string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // We use the same parser as `PackageDependencyAtom` since `PackageAtom`
        // is just a subset.
        let atom = PackageDependencyParser::parse_atom(s)?;

        ensure!(
            atom.block == PackageBlock::None,
            "Blockers are invalid in this context."
        );

        ensure!(
            atom.uses.is_empty(),
            "USE constraints are invalid in this context."
        );

        let slot = if let Some(slot) = atom.slot {
            ensure!(
                !slot.rebuild_on_slot_change,
                "Slot operators are invalid in this context"
            );

            if let Some(slot) = slot.slot {
                Some(slot)
            } else {
                bail!("Slot name is required");
            }
        } else {
            None
        };

        Ok(PackageAtom {
            package_name: atom.package_name,
            version: atom.version,
            slot,
        })
    }
}

impl Display for PackageAtom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(version) = &self.version {
            write!(f, "{}", version.op)?;
        }
        write!(f, "{}", &self.package_name)?;
        if let Some(version) = &self.version {
            write!(f, "-{}", version.version)?;
            if let PackageVersionOp::Equal { wildcard: true } = version.op {
                write!(f, "*")?;
            }
        }
        if let Some((slot, subslot)) = &self.slot {
            write!(f, ":{}", slot)?;
            if let Some(subslot) = subslot {
                write!(f, "/{}", subslot)?;
            }
        }
        Ok(())
    }
}

impl Predicate<PackageRef<'_>> for PackageAtom {
    fn matches(&self, package: &PackageRef<'_>) -> bool {
        if package.package_name != self.package_name {
            return false;
        }
        if let Some(p) = &self.version {
            if !p.matches(package.version) {
                return false;
            }
        }
        if let Some((slot, subslot)) = &self.slot {
            if slot != package.slot.main {
                return false;
            }

            if let Some(subslot) = subslot {
                if subslot != package.slot.sub {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, str::FromStr};

    use anyhow::Result;

    use super::*;

    #[test]
    fn test_parse_package_atom() -> Result<()> {
        let test_cases = HashMap::from([
            (
                "sys-apps/systemd-utils",
                PackageAtom {
                    package_name: "sys-apps/systemd-utils".to_owned(),
                    version: None,
                    slot: None,
                },
            ),
            (
                "=sys-apps/systemd-utils-9999",
                PackageAtom {
                    package_name: "sys-apps/systemd-utils".to_owned(),
                    version: Some(PackageVersionDependency {
                        op: PackageVersionOp::Equal { wildcard: false },
                        version: Version::try_new("9999")?,
                    }),
                    slot: None,
                },
            ),
            (
                "=sys-apps/systemd-utils-1*",
                PackageAtom {
                    package_name: "sys-apps/systemd-utils".to_owned(),
                    version: Some(PackageVersionDependency {
                        op: PackageVersionOp::Equal { wildcard: true },
                        version: Version::try_new("1")?,
                    }),
                    slot: None,
                },
            ),
            (
                "~sys-apps/systemd-utils-1",
                PackageAtom {
                    package_name: "sys-apps/systemd-utils".to_owned(),
                    version: Some(PackageVersionDependency {
                        op: PackageVersionOp::EqualExceptRevision,
                        version: Version::try_new("1")?,
                    }),
                    slot: None,
                },
            ),
            (
                "sys-apps/systemd-utils:1",
                PackageAtom {
                    package_name: "sys-apps/systemd-utils".to_owned(),
                    version: None,
                    slot: Some(("1".to_string(), None)),
                },
            ),
            (
                "sys-apps/systemd-utils:1/2",
                PackageAtom {
                    package_name: "sys-apps/systemd-utils".to_owned(),
                    version: None,
                    slot: Some(("1".to_string(), Some("2".to_string()))),
                },
            ),
        ]);

        for (input, expected) in test_cases {
            let actual = PackageAtom::from_str(input)?;

            assert_eq!(expected, actual, "input: {}", input);
            assert_eq!(input, format!("{}", actual));
        }

        Ok(())
    }

    #[test]
    fn test_parse_package_atom_invalid() -> Result<()> {
        let test_cases = vec![
            "=sys-apps/systemd-utils",
            "=sys-apps/systemd-utils-",
            "sys-apps/systemd-utils:=",
            "sys-apps/systemd-utils:*",
            "sys-apps/systemd-utils:1=",
            "sys-apps/systemd-utils[udev]",
        ];

        for input in test_cases {
            let result = PackageAtom::from_str(input);

            assert!(result.is_err(), "input: {}", input);
        }

        Ok(())
    }

    #[test]
    fn test_parse_package_atom_match() -> Result<()> {
        let package = PackageRef {
            package_name: "sys-apps/systemd-utils",
            version: &Version::try_new("9999")?,
            slot: Slot {
                main: "1",
                sub: "2",
            },
            use_map: &UseMap::new(),
        };

        let test_cases = HashMap::from([
            ("sys-apps/systemd-utils", true),
            ("=sys-apps/systemd-utils-9999", true),
            // TODO(b/272798056): Wildcard matching doesn't quite work right.
            // ("=sys-apps/systemd-utils-9*", true),
            ("=sys-apps/systemd-utils-1*", false),
            ("~sys-apps/systemd-utils-1", false),
            ("sys-apps/systemd-utils:1", true),
            ("sys-apps/systemd-utils:2", false),
            ("sys-apps/systemd-utils:1/2", true),
            ("sys-apps/systemd-utils:1/4", false),
        ]);

        for (input, expected) in test_cases {
            let atom = PackageAtom::from_str(input)?;
            let actual = atom.matches(&package);

            assert_eq!(expected, actual, "input: {}", input);
        }

        Ok(())
    }
}
