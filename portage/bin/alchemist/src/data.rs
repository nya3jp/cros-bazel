// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Deref, DerefMut},
};

/// A dictionary of variables defined in Portage configurations, such as
/// `make.conf` and profiles.
pub type Vars = HashMap<String, String>;

/// Represents USE flags disabled/enabled for a particular package.
///
/// It should contain all USE flags defined explicitly in IUSE, and those
/// available implicitly due to profile-injected IUSE (aka IUSE_EFFECTIVE in
/// PMS). Other USE flags are omitted/hidden in the map.
///
/// This is not an alias of [`HashMap`] because we need to implement
/// [`AsRef<UseMap>`]. Unfortunately [`AsRef`] is not reflexive:
/// https://doc.rust-lang.org/std/convert/trait.AsRef.html#reflexivity.
/// We though try to make it behave like a plain [`HashMap`] by implementing
/// [`Deref`] and [`DerefMut`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UseMap(HashMap<String, bool>);

impl UseMap {
    pub fn new() -> Self {
        UseMap(HashMap::new())
    }
}

impl<T: Into<HashMap<String, bool>>> From<T> for UseMap {
    fn from(value: T) -> Self {
        UseMap(value.into())
    }
}

impl FromIterator<(String, bool)> for UseMap {
    fn from_iter<T: IntoIterator<Item = (String, bool)>>(iter: T) -> Self {
        UseMap(HashMap::from_iter(iter))
    }
}

impl Deref for UseMap {
    type Target = HashMap<String, bool>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UseMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<UseMap> for UseMap {
    fn as_ref(&self) -> &UseMap {
        self
    }
}

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
