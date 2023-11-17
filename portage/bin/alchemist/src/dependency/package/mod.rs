// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

pub mod parser;

use anyhow::{bail, ensure, Context, Error, Result};
use itertools::Itertools;
use std::{collections::BTreeMap, fmt::Display, str::FromStr, sync::Arc};
use version::Version;

use crate::{
    config::ProvidedPackage,
    data::{Slot, UseMap},
};

use super::{Dependency, DependencyMeta, Predicate};
use parser::PackageDependencyParser;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageDependencyMeta;

impl DependencyMeta for PackageDependencyMeta {
    type Leaf = PackageDependencyAtom;
    type Parser = PackageDependencyParser;
}

/// Alias of Dependency specialized to package dependencies.
pub type PackageDependency = Dependency<PackageDependencyMeta>;

/// A borrowed subset of package data to be passed to package-related predicates.
#[derive(Clone, Copy, Debug)]
pub struct PackageRef<'a> {
    // Package name and version are required because they should be always available.
    pub package_name: &'a str,
    pub version: &'a Version,

    // Remaining fields are optional. They're matched only when they're available.
    pub slot: Option<Slot<&'a str>>,
    pub use_map: Option<&'a UseMap>,
}

/// A trait to be implemented by types that can be converted to [`PackageRef`].
pub trait AsPackageRef {
    fn as_package_ref(&self) -> PackageRef;
}

impl<T: AsPackageRef> AsPackageRef for &T {
    fn as_package_ref(&self) -> PackageRef {
        (*self).as_package_ref()
    }
}

impl<T: AsPackageRef> AsPackageRef for Arc<T> {
    fn as_package_ref(&self) -> PackageRef {
        (**self).as_package_ref()
    }
}

/// Holds a set of [`PackageRef`].
#[derive(Debug)]
pub struct PackageRefSet<'a> {
    // Keys are package names.
    packages: BTreeMap<&'a str, Vec<PackageRef<'a>>>,
}

impl<'a> PackageRefSet<'a> {
    pub fn get(&self, package_name: &str) -> &[PackageRef<'a>] {
        self.packages
            .get(package_name)
            .map(|v| v.as_slice())
            .unwrap_or_default()
    }
}

impl<'a> FromIterator<PackageRef<'a>> for PackageRefSet<'a> {
    fn from_iter<T: IntoIterator<Item = PackageRef<'a>>>(iter: T) -> Self {
        let mut packages: BTreeMap<&str, Vec<PackageRef>> = BTreeMap::new();
        for package in iter {
            packages
                .entry(package.package_name)
                .or_default()
                .push(package);
        }
        Self { packages }
    }
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

    pub fn matches(&self, slot: &Slot<&str>) -> bool {
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

impl PackageUseDependency {
    fn matches(&self, source_use_map: &UseMap, target_use_map: &UseMap) -> Result<bool> {
        let target_value = target_use_map
            .get(&self.flag)
            .copied()
            .or(self.missing_default)
            .with_context(|| {
                format!(
                    "Target is missing '{}' USE flag and no default specified",
                    self.flag
                )
            })?;

        let result = match self.op {
            PackageUseDependencyOp::Required => target_value ^ self.negate,
            PackageUseDependencyOp::Synchronized => {
                let source_value = source_use_map
                    .get(&self.flag)
                    .copied()
                    .with_context(|| format!("Missing source USE flag '{}'", self.flag))?;

                source_value == (target_value ^ self.negate)
            }
            PackageUseDependencyOp::ConditionalRequired => {
                let source_value = source_use_map
                    .get(&self.flag)
                    .copied()
                    .with_context(|| format!("Missing source USE flag '{}'", self.flag))?;

                if self.negate {
                    source_value || !target_value
                } else {
                    !source_value || target_value
                }
            }
        };
        Ok(result)
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

    pub fn matches(&self, version: &Version) -> bool {
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

impl PackageDependencyAtom {
    fn matches_ignoring_block(
        &self,
        source_use_map: &UseMap,
        package: &PackageRef,
    ) -> Result<bool> {
        if package.package_name != self.package_name {
            return Ok(false);
        }

        if let Some(p) = &self.version {
            if !p.matches(package.version) {
                return Ok(false);
            }
        }

        if let Some(slot) = &package.slot {
            if let Some(p) = &self.slot {
                if !p.matches(slot) {
                    return Ok(false);
                }
            }
        }

        if let Some(use_map) = &package.use_map {
            for uses in &self.uses {
                if !uses.matches(source_use_map, use_map)? {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
}

impl Predicate<PackageRef<'_>> for PackageDependencyAtom {
    fn matches(&self, source_use_map: &UseMap, package: &PackageRef) -> Result<bool> {
        // TODO: Introduce a type that is similar to `PackageDependencyAtom` but guaranteed to
        // contain no block, and move this method to the type so that we can avoid this error.
        if self.block != PackageBlock::None {
            bail!("BUG: Blocking package dependency can't be matched with a single package atom");
        }
        self.matches_ignoring_block(source_use_map, package)
    }
}

impl Predicate<PackageRefSet<'_>> for PackageDependencyAtom {
    fn matches(&self, source_use_map: &UseMap, packages: &PackageRefSet) -> Result<bool> {
        let mut any_match = false;
        for package in packages.get(&self.package_name) {
            if self.matches_ignoring_block(source_use_map, package)? {
                any_match = true;
            }
        }

        // If the atom is not a blocker, it's considered satisfied iff any package matches it.
        // If the atom is a blocker, it's considered satisfied iff no package matches it.
        Ok(any_match == (self.block == PackageBlock::None))
    }
}

impl PackageDependencyAtom {
    /// Checks if the [`ProvidedPackage`] matches the `atom`.
    ///
    /// A ProvidedPackage only contains a package_name and a version. This
    /// unfortunately means we can't match against `slot` or `USE` dependencies.
    /// These constraints are ignored when matching against a provided package.
    ///
    /// Due to these limitations, the EAPI7 has deprecated and strongly
    /// discourages the use of package.provided.
    pub fn matches_provided(&self, package: &ProvidedPackage) -> bool {
        // TODO: Introduce a type that is similar to `PackageDependencyAtom` but guaranteed to
        // contain no block, and move this method to the type so that we can avoid this panic.
        if self.block != PackageBlock::None {
            panic!("BUG: Blocking package dependency can't be matched with package.provided");
        }

        if package.package_name != self.package_name {
            return false;
        }

        if let Some(p) = &self.version {
            if !p.matches(&package.version) {
                return false;
            }
        }
        true
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

impl PackageAtom {
    pub fn matches(&self, package: &PackageRef) -> bool {
        if package.package_name != self.package_name {
            return false;
        }

        if let Some(p) = &self.version {
            if !p.matches(package.version) {
                return false;
            }
        }

        if let Some(package_slot) = &package.slot {
            if let Some((slot, subslot)) = &self.slot {
                if slot != package_slot.main {
                    return false;
                }

                if let Some(subslot) = subslot {
                    if subslot != package_slot.sub {
                        return false;
                    }
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, str::FromStr};

    use anyhow::{anyhow, Result};

    use crate::dependency::ThreeValuedPredicate;

    use super::*;

    #[test]
    fn test_package_dependency_matches() -> Result<()> {
        let empty_use_map = UseMap::new();
        let default_version = Version::try_new("1.0").unwrap();
        let package_set = PackageRefSet::from_iter([
            PackageRef {
                package_name: "pkg/aaa",
                version: &default_version,
                slot: Some(Slot::new("0")),
                use_map: Some(&empty_use_map),
            },
            PackageRef {
                package_name: "pkg/bbb",
                version: &default_version,
                slot: Some(Slot::new("0")),
                use_map: Some(&empty_use_map),
            },
        ]);

        let test_cases = [
            ("", true),
            ("pkg/aaa", true),
            ("pkg/aaa pkg/bbb", true),
            ("pkg/aaa pkg/xxx", false),
            ("pkg/aaa || ( pkg/xxx pkg/bbb )", true),
            ("pkg/aaa !pkg/bbb", false),
            ("|| ( pkg/aaa !pkg/bbb )", true),
            ("|| ( !pkg/aaa !pkg/bbb )", false),
        ];

        for (raw_deps, want) in test_cases {
            let deps = PackageDependency::from_str(raw_deps)?;
            assert_eq!(
                deps.matches(&empty_use_map, &package_set)?,
                Some(want),
                "matches({:?})",
                raw_deps
            );
        }

        Ok(())
    }

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
            slot: Some(Slot {
                main: "1",
                sub: "2",
            }),
            use_map: None,
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

    #[test]
    fn test_use_match() -> Result<()> {
        let test_cases: Vec<(&str, UseMap, UseMap, Result<bool>)> = vec![
            // Simple flag
            (
                "udev",
                UseMap::new(),
                UseMap::from([("udev".to_string(), true)]),
                Ok(true),
            ),
            (
                "udev",
                UseMap::new(),
                UseMap::from([("udev".to_string(), false)]),
                Ok(false),
            ),
            (
                "udev",
                UseMap::new(),
                UseMap::new(),
                Err(anyhow!("Missing target USE flag")),
            ),
            // Simple negate
            (
                "-udev",
                UseMap::new(),
                UseMap::from([("udev".to_string(), true)]),
                Ok(false),
            ),
            (
                "-udev",
                UseMap::new(),
                UseMap::from([("udev".to_string(), false)]),
                Ok(true),
            ),
            (
                "-udev",
                UseMap::new(),
                UseMap::new(),
                Err(anyhow!("Missing target USE flag")),
            ),
            // Default (+) ignored
            (
                "udev(+)",
                UseMap::new(),
                UseMap::from([("udev".to_string(), true)]),
                Ok(true),
            ),
            (
                "udev(+)",
                UseMap::new(),
                UseMap::from([("udev".to_string(), false)]),
                Ok(false),
            ),
            (
                "-udev(+)",
                UseMap::new(),
                UseMap::from([("udev".to_string(), true)]),
                Ok(false),
            ),
            (
                "-udev(+)",
                UseMap::new(),
                UseMap::from([("udev".to_string(), false)]),
                Ok(true),
            ),
            // Default (-) ignored
            (
                "udev(-)",
                UseMap::new(),
                UseMap::from([("udev".to_string(), true)]),
                Ok(true),
            ),
            (
                "udev(-)",
                UseMap::new(),
                UseMap::from([("udev".to_string(), false)]),
                Ok(false),
            ),
            (
                "-udev(-)",
                UseMap::new(),
                UseMap::from([("udev".to_string(), true)]),
                Ok(false),
            ),
            (
                "-udev(-)",
                UseMap::new(),
                UseMap::from([("udev".to_string(), false)]),
                Ok(true),
            ),
            // Default (+)
            ("udev(+)", UseMap::new(), UseMap::new(), Ok(true)),
            ("-udev(+)", UseMap::new(), UseMap::new(), Ok(false)),
            // Default (-)
            ("udev(-)", UseMap::new(), UseMap::new(), Ok(false)),
            ("-udev(-)", UseMap::new(), UseMap::new(), Ok(true)),
            // Synchronized
            (
                "udev=",
                UseMap::from([("udev".to_string(), true)]),
                UseMap::from([("udev".to_string(), true)]),
                Ok(true),
            ),
            (
                "udev=",
                UseMap::from([("udev".to_string(), false)]),
                UseMap::from([("udev".to_string(), false)]),
                Ok(true),
            ),
            (
                "udev=",
                UseMap::from([("udev".to_string(), true)]),
                UseMap::from([("udev".to_string(), false)]),
                Ok(false),
            ),
            (
                "udev=",
                UseMap::from([("udev".to_string(), false)]),
                UseMap::from([("udev".to_string(), true)]),
                Ok(false),
            ),
            (
                "udev=",
                UseMap::new(),
                UseMap::from([("udev".to_string(), true)]),
                Err(anyhow!("Missing source USE flag")),
            ),
            // Synchronized not
            (
                "!udev=",
                UseMap::from([("udev".to_string(), true)]),
                UseMap::from([("udev".to_string(), true)]),
                Ok(false),
            ),
            (
                "!udev=",
                UseMap::from([("udev".to_string(), false)]),
                UseMap::from([("udev".to_string(), false)]),
                Ok(false),
            ),
            (
                "!udev=",
                UseMap::from([("udev".to_string(), true)]),
                UseMap::from([("udev".to_string(), false)]),
                Ok(true),
            ),
            (
                "!udev=",
                UseMap::from([("udev".to_string(), false)]),
                UseMap::from([("udev".to_string(), true)]),
                Ok(true),
            ),
            (
                "!udev=",
                UseMap::new(),
                UseMap::from([("udev".to_string(), true)]),
                Err(anyhow!("Missing source USE flag")),
            ),
            // Conditionally required
            (
                "udev?",
                UseMap::from([("udev".to_string(), true)]),
                UseMap::from([("udev".to_string(), true)]),
                Ok(true),
            ),
            (
                "udev?",
                UseMap::from([("udev".to_string(), false)]),
                UseMap::from([("udev".to_string(), false)]),
                Ok(true),
            ),
            (
                "udev?",
                UseMap::from([("udev".to_string(), true)]),
                UseMap::from([("udev".to_string(), false)]),
                Ok(false),
            ),
            (
                "udev?",
                UseMap::from([("udev".to_string(), false)]),
                UseMap::from([("udev".to_string(), true)]),
                Ok(true),
            ),
            (
                "udev?",
                UseMap::new(),
                UseMap::from([("udev".to_string(), true)]),
                Err(anyhow!("Missing source USE flag")),
            ),
            // Conditionally required not
            (
                "!udev?",
                UseMap::from([("udev".to_string(), true)]),
                UseMap::from([("udev".to_string(), true)]),
                Ok(true),
            ),
            (
                "!udev?",
                UseMap::from([("udev".to_string(), false)]),
                UseMap::from([("udev".to_string(), false)]),
                Ok(true),
            ),
            (
                "!udev?",
                UseMap::from([("udev".to_string(), true)]),
                UseMap::from([("udev".to_string(), false)]),
                Ok(true),
            ),
            (
                "!udev?",
                UseMap::from([("udev".to_string(), false)]),
                UseMap::from([("udev".to_string(), true)]),
                Ok(false),
            ),
            (
                "!udev?",
                UseMap::new(),
                UseMap::from([("udev".to_string(), true)]),
                Err(anyhow!("Missing source USE flag")),
            ),
        ];

        for (input, source_use, target_use, expected) in test_cases {
            // We only want to unit test the USE matching
            let mut uses =
                PackageDependencyAtom::from_str(&format!("sys-apps/systemd-utils[{input}]"))?
                    .uses
                    .into_iter();
            let atom = uses.next().unwrap();
            assert!(uses.next().is_none());

            let actual = atom.matches(&source_use, &target_use);

            match expected {
                Ok(expected) => {
                    assert_eq!(
                        expected,
                        actual.unwrap(),
                        "input: {}, source_use: {:?}, target_use: {:?}",
                        input,
                        source_use,
                        target_use
                    );
                }
                Err(_) => {
                    assert!(actual.is_err(), "input: {}", input);
                }
            }
        }

        Ok(())
    }

    #[test]
    fn test_all_use_match() -> Result<()> {
        let atom = PackageDependencyAtom::from_str("sys-apps/systemd-utils[udev,-boot]")?;

        {
            let use_map = UseMap::from([("udev".to_string(), true), ("boot".to_string(), false)]);
            let package = PackageRef {
                package_name: "sys-apps/systemd-utils",
                version: &Version::try_new("9999")?,
                slot: Some(Slot {
                    main: "1",
                    sub: "2",
                }),
                use_map: Some(&use_map),
            };

            assert!(atom.matches(&UseMap::new(), &package)?);
        }

        {
            let use_map = UseMap::from([("udev".to_string(), true), ("boot".to_string(), true)]);
            let package = PackageRef {
                package_name: "sys-apps/systemd-utils",
                version: &Version::try_new("9999")?,
                slot: Some(Slot {
                    main: "1",
                    sub: "2",
                }),
                use_map: Some(&use_map),
            };

            assert!(!atom.matches(&UseMap::new(), &package)?);
        }

        Ok(())
    }

    #[test]
    fn test_parse_package_dependency_atom_match_block() -> Result<()> {
        let use_map = UseMap::new();
        let package = PackageRef {
            package_name: "sys-apps/attr",
            version: &Version::try_new("9999")?,
            slot: Some(Slot::new("0")),
            use_map: Some(&use_map),
        };

        let atom = PackageDependencyAtom::from_str("!sys-apps/acl")?;
        atom.matches(&UseMap::new(), &package)
            .expect_err("Must fail for block dependencies");

        Ok(())
    }
}
