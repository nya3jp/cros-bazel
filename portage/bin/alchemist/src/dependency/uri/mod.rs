// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::fmt::Display;
use url::Url;

use self::parser::UriDependencyParser;

use super::{Dependency, DependencyMeta};

mod parser;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UriDependencyMeta;

impl DependencyMeta for UriDependencyMeta {
    type Leaf = UriAtomDependency;
    type Parser = UriDependencyParser;
}

/// Alias of Dependency specialized to URI dependencies.
pub type UriDependency = Dependency<UriDependencyMeta>;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UriAtomDependency {
    Uri(Url, Option<String>),
    Filename(String),
}

impl Display for UriAtomDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uri(url, filename) => {
                write!(f, "{}", url)?;
                if let Some(filename) = filename {
                    write!(f, " -> {}", filename)?;
                }
                Ok(())
            }
            Self::Filename(filename) => {
                write!(f, "{}", filename)
            }
        }
    }
}
