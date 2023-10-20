// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use self::parser::RestrictDependencyParser;

use super::Dependency;
use super::DependencyParserType;

mod parser;

/// Alias of Dependency specialized to package RESTRICT options.
pub type RestrictDependency = Dependency<RestrictAtom>;

/// See man 5 ebuild
#[derive(
    Clone,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    strum_macros::Display,
    strum_macros::EnumString,
)]
pub enum RestrictAtom {
    /// Disable all QA checks for binaries.
    #[strum(serialize = "binchecks")]
    BinChecks,

    /// Distribution of built packages is restricted.
    #[strum(serialize = "bindist")]
    BinDist,

    /// Like mirror but the files will not be fetched via SRC_URI either.
    #[strum(serialize = "fetch")]
    Fetch,

    /// Disables installsources for specific packages.
    #[strum(serialize = "installsources")]
    InstallSources,

    /// Files in SRC_URI will not be downloaded from the GENTOO_MIRRORS.
    #[strum(
        serialize = "mirror",

        // Portage also supports this to work around some bad ebuilds
        // TODO(b/271845160): Remove this once we rebase on ToT
        serialize = "nomirror"
    )]
    Mirror,

    /// Disables the network namespace for a specific package.
    #[strum(serialize = "network-sandbox")]
    NetworkSandbox,

    /// Disables preserve-libs for specific packages.
    #[strum(serialize = "preserve-libs")]
    PreserveLibs,

    /// Fetch from URIs in SRC_URI before GENTOO_MIRRORS.
    #[strum(serialize = "primaryuri")]
    PrimaryUri,

    /// Disables splitdebug for specific packages.
    #[strum(serialize = "splitdebug")]
    SplitDebug,

    /// Final binaries/libraries will not be stripped of debug symbols.
    #[strum(serialize = "strip")]
    Strip,

    /// Do not run src_test even if user has FEATURES=test.
    #[strum(serialize = "test")]
    Test,

    /// Disables userpriv for specific packages.
    #[strum(serialize = "userpriv")]
    UserPriv,
}

impl DependencyParserType<RestrictAtom> for RestrictAtom {
    type Parser = RestrictDependencyParser;
}
