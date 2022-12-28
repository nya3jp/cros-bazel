// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::fmt::Display;
use url::Url;

use super::{
    parser::{uri::UriDependencyParser, DependencyParserType},
    Dependency,
};

/// Alias of Dependency specialized to URI dependencies.
pub type UriDependency = Dependency<UriAtomDependency>;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UriAtomDependency {
    Uri(Url, Option<String>),
    Filename(String),
}

impl DependencyParserType<UriAtomDependency> for UriAtomDependency {
    type Parser = UriDependencyParser;
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
