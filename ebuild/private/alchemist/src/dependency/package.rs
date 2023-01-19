// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Error, Result};
use itertools::Itertools;
use std::{fmt::Display, str::FromStr};
use version::Version;

use crate::data::{Slot, UseMap};

use super::{
    parser::{package::PackageDependencyParser, DependencyParserType},
    Dependency, Predicate,
};

/// Alias of Dependency specialized to package dependencies.
pub type PackageDependency = Dependency<PackageAtomDependency>;

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
/// that are available before evaluating ebuild metadata.
///
/// We use this struct to work with package dependency atoms evaluated before
/// package metadata generation, e.g. on processing `package.use`.
#[derive(Clone, Copy, Debug)]
pub struct ThinPackageRef<'a> {
    pub package_name: &'a str,
    pub version: &'a Version,
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

/// Represents a package USE dependency.
///
/// This is a subcomponent of [`PackageAtomDependency`].
///
/// See the PMS for the specification:
/// https://projects.gentoo.org/pms/8/pms.html#x1-830008.3.4
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackageUseDependency {
    raw: String,
}

impl PackageUseDependency {
    pub(super) fn new(raw: String) -> Self {
        Self { raw }
    }
}

impl Predicate<UseMap> for PackageUseDependency {
    fn matches(&self, _uses: &UseMap) -> bool {
        // TODO: Implement USE dependencies.
        true
    }
}

impl Display for PackageUseDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.raw)
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
/// See the PMS for the specification:
/// https://projects.gentoo.org/pms/8/pms.html#x1-790008.3
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PackageAtomDependency {
    package_name: String,
    version: Option<PackageVersionDependency>,
    slot: Option<PackageSlotDependency>,
    uses: Vec<PackageUseDependency>,
    block: PackageBlock,
}

impl PackageAtomDependency {
    pub(super) fn new(
        package_name: String,
        version: Option<PackageVersionDependency>,
        slot: Option<PackageSlotDependency>,
        uses: Vec<PackageUseDependency>,
        block: PackageBlock,
    ) -> Self {
        Self {
            package_name,
            version,
            slot,
            uses,
            block,
        }
    }

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

impl FromStr for PackageAtomDependency {
    type Err = Error;

    /// Parses a package dependency atom string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PackageDependencyParser::parse_atom(s)
    }
}

impl DependencyParserType<PackageAtomDependency> for PackageAtomDependency {
    type Parser = PackageDependencyParser;
}

impl Display for PackageAtomDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.block)?;
        if let Some(version) = &self.version {
            write!(f, "{}", version.op)?;
        }
        write!(f, "{}", &self.package_name)?;
        if let Some(version) = &self.version {
            write!(f, "-{}", version.version)?;
            match version.op {
                PackageVersionOp::Equal { wildcard: true } => {
                    write!(f, "*")?;
                }
                _ => {}
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

impl Predicate<PackageRef<'_>> for PackageAtomDependency {
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

impl Predicate<ThinPackageRef<'_>> for PackageAtomDependency {
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
            true
        })();

        match_except_block == (self.block == PackageBlock::None)
    }
}
