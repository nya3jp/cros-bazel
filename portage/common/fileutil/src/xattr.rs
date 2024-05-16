// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// Enumerates all user xattrs of a file.
pub fn list_user_xattrs(path: &Path) -> Result<Vec<OsString>> {
    let mut keys: Vec<OsString> = Vec::new();
    for key in xattr::list(path)? {
        if key.to_string_lossy().starts_with("user.") {
            keys.push(key);
        }
    }
    Ok(keys)
}

/// Returns all user xattrs of a file as a [`BTreeMap`].
pub fn get_user_xattrs_map(path: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut xattrs: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for raw_key in list_user_xattrs(path)? {
        let value = xattr::get(path, &raw_key)?.unwrap_or_default();
        let key = String::from_utf8(raw_key.as_bytes().to_owned())?;
        xattrs.insert(key, value);
    }
    Ok(xattrs)
}
