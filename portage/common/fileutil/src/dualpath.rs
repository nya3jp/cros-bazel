// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Eq, PartialEq)]
pub struct DualPath {
    pub outside: PathBuf,
    pub inside: PathBuf,
}

impl DualPath {
    pub fn join<P: AsRef<Path>>(&self, path: P) -> DualPath {
        Self {
            outside: self.outside.join(&path),
            inside: self.inside.join(&path),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dualpath_join() {
        let dp = DualPath {
            inside: PathBuf::from("/inside"),
            outside: PathBuf::from("/outside"),
        };
        assert_eq!(
            dp.join("blah"),
            DualPath {
                inside: PathBuf::from("/inside/blah"),
                outside: PathBuf::from("/outside/blah"),
            }
        );
    }
}
