// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{collections::HashMap, fmt::Display};

/// A dictionary of variables defined in Portage configurations, such as
/// `make.conf` and profiles.
pub type Vars = HashMap<String, String>;

/// Represents USE flags disabled/enabled for a particular package.
/// It should contain all USE flags defined explicitly in IUSE, and those
/// available implicitly due to profile-injected IUSE (aka IUSE_EFFECTIVE in
/// PMS). Other USE flags are omitted/hidden in the map.
pub type UseMap = HashMap<String, bool>;

/// Represents IUSE declared by a package. A value in the map indicates the
/// default value of a USE flag to be used if it is not set in profiles.
pub type IUseMap = HashMap<String, bool>;

/// Represents SLOT declared by a package.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Slot<S = String> {
    pub main: S,
    pub sub: S,
}

impl<'s, S: From<&'s str>> Slot<S> {
    pub fn new(s: &'s (impl AsRef<str> + ?Sized)) -> Self {
        let s = s.as_ref();
        let (main, sub) = s.split_once('/').unwrap_or((s, s));
        Self {
            main: main.into(),
            sub: sub.into(),
        }
    }
}

impl<S: Display> Display for Slot<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", &self.main, &self.sub)
    }
}

/// A pair of a package name and a (main) SLOT, for which at most single package
/// can be selected for installation. This is to be used as a key of package
/// collections when selecting packages.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackageSlotKey {
    pub package_name: String,
    pub main_slot: String,
}
