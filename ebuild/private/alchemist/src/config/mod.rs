// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

pub mod bundle;
pub mod makeconf;
pub mod miscconf;
pub mod profile;
pub mod site;

use std::path::{Path, PathBuf};

use version::Version;

use crate::{
    data::Vars,
    dependency::package::{PackageDependencyAtom, ThinPackageRef},
};

/// Represents a kind of a USE flag update entry.
///
/// This is a field of [`UseUpdate`].
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UseUpdateKind {
    /// Sets the enabled/disabled state of a USE flag.
    /// Updates of this kind are configured in `package.use` for example.
    Set,
    /// Sets the masked/unmasked state of a USE flag.
    /// Updates of this kind are configured in `package.use.mask` for example.
    Mask,
    /// Sets the forced/unforced state of a USE flag.
    /// Updates of this kind are configured in `package.use.force` for example.
    Force,
}

/// Represents the targets of a USE flag update entry.
///
/// This is a field of [`UseUpdate`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UseUpdateFilter {
    /// Specifies the packages affected by this update.
    /// If it is [None], the update applies to all packages.
    /// This is unset for configurations such as `use.mask`; set for others
    /// such as `package.use.mask`.
    pub atom: Option<PackageDependencyAtom>,
    /// If it is true, this update applies to stable packages only.
    /// This is set for configurations such as `package.use.stable`.
    pub stable_only: bool,
}

/// Represents an update of a USE flag state.
///
/// A USE flag can be updated in many ways:
/// - `package.use` may update enabled/disabled state of USE flags for certain
///   packages.
/// - `use.mask` and `use.stable.mask` may update masked/unmasked state of USE
///   flags globally.
/// - `package.use.mask` and `package.use.stable.mask` may update
///   masked/unmasked state of USE flags for certain packages.
/// - `use.force`, `use.stable.force`, `package.use.force`, and
///   `package.use.stable.force` may update forced/unforced state of USE
///   flags.
///
/// This struct represents an entry of these updates in a common format.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UseUpdate {
    pub kind: UseUpdateKind,
    pub filter: UseUpdateFilter,
    pub use_tokens: String,
}

/// Represents a kind of a package mask entry.
///
/// This is a field of [`PackageMaskUpdate`].
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PackageMaskKind {
    /// Masks a package.
    Mask,
    /// Unmasks a package.
    Unmask,
}

/// Represents an update of a package mask state.
///
/// A package can be masked by `package.mask` and `package.unmask`.
/// This struct represents an entry of these updates in a common format.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageMaskUpdate {
    pub kind: PackageMaskKind,
    pub atom: PackageDependencyAtom,
}

/// Represents a package pretended as provided.
///
/// A package can be pretended as provided by `package.provided`.
/// This struct represents such an entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvidedPackage {
    pub package_name: String,
    pub version: Version,
}

impl ProvidedPackage {
    pub fn as_thin_package_ref(&self) -> ThinPackageRef<'_> {
        ThinPackageRef {
            package_name: &self.package_name,
            version: &self.version,
        }
    }
}

/// Configurations provided by a [`ConfigNode`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigNodeValue {
    /// Provides variables from a `make.conf`-style configuration file.
    /// This contains variable values just as they're defined in a file, which
    /// means that incremental variables are not yet resolved.
    Vars(Vars),
    /// Updates USE flags.
    Uses(Vec<UseUpdate>),
    /// Updates package masks.
    PackageMasks(Vec<PackageMaskUpdate>),
    /// Updates provided packages.
    ProvidedPackages(Vec<ProvidedPackage>),
}

/// Represents a node in Portage configurations.
///
/// Portage configurations can be represented as an ordered list of
/// configuration nodes, which can be evaluated by processing each node in the
/// order. [`ConfigNode`] represents a single entry in the list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigNode {
    /// Path to the file that provided this configuration node.
    pub source: PathBuf,

    /// Actual configurations provided in the node.
    pub value: ConfigNodeValue,
}

impl ConfigNode {
    pub fn new(source: &Path, value: ConfigNodeValue) -> Self {
        Self {
            source: source.to_owned(),
            value,
        }
    }
}

/// Source of [`ConfigNode`]s.
///
/// Portage reads configurations from various sources, such as `make.conf`,
/// profiles, and environmental variables. This trait abstracts such a
/// configuration source.
pub trait ConfigSource {
    fn evaluate_configs(&self, env: &mut Vars) -> Vec<ConfigNode>;
}

impl<S: ConfigSource + ?Sized> ConfigSource for Box<S> {
    fn evaluate_configs(&self, env: &mut Vars) -> Vec<ConfigNode> {
        (**self).evaluate_configs(env)
    }
}

/// An implementation of [`ConfigSource`] that simply returns [`ConfigNode`]s
/// that are set on its constructor.
///
/// This is useful for overriding configurations for hacks.
pub struct SimpleConfigSource {
    nodes: Vec<ConfigNode>,
}

impl SimpleConfigSource {
    pub fn new(nodes: Vec<ConfigNode>) -> Self {
        Self { nodes }
    }
}

impl ConfigSource for SimpleConfigSource {
    fn evaluate_configs(&self, env: &mut Vars) -> Vec<ConfigNode> {
        for node in self.nodes.iter() {
            if let ConfigNodeValue::Vars(vars) = &node.value {
                env.extend(vars.clone());
            }
        }
        self.nodes.clone()
    }
}
