// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use log::debug;
use nix::unistd::{Gid, Group, Uid, User};
use serde::Deserialize;
use std::collections::HashSet;
use std::collections::{
    hash_map, {BTreeMap, HashMap},
};
use std::fs::File;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

fn get_install_path(install_dir: &Path, p: &Path) -> PathBuf {
    // Absolute paths here should be ignored. If I say to install at
    // "/usr/bin/foo" I really want to install at "install_dir/usr/bin/foo"
    install_dir.join(p.strip_prefix("/").unwrap_or(p))
}

struct Cache {
    pub uid_map: HashMap<String, Uid>,
    pub gid_map: HashMap<String, Gid>,
    pub runfiles: runfiles::Runfiles,
    created_dirs: HashSet<PathBuf>,
}

impl Cache {
    pub fn create() -> Result<Self> {
        Ok(Cache {
            uid_map: HashMap::new(),
            gid_map: HashMap::new(),
            runfiles: runfiles::Runfiles::create()?,
            created_dirs: HashSet::new(),
        })
    }

    pub fn create_dir(&mut self, p: &Path) -> Result<()> {
        let p = PathBuf::from(p);
        if !self.created_dirs.contains(&p) {
            std::fs::create_dir_all(&p)?;
            self.created_dirs.insert(p);
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
struct Attributes {
    mode: String,
    uid: Option<u32>,
    gid: Option<u32>,
    user: Option<String>,
    group: Option<String>,
}

fn get_id<Id: Clone + std::fmt::Display + PartialEq<Id>, T>(
    map: &mut HashMap<String, Id>,
    id: Option<Id>,
    name: &Option<String>,
    f: impl FnOnce(&str) -> Result<Option<T>>,
    getter: impl FnOnce(T) -> Id,
    kind: &str,
) -> Result<Option<Id>> {
    match (id, name) {
        (None, None) => Ok(None),
        (Some(id), None) => Ok(Some(id)),
        (id, Some(name)) => {
            let id_from_name = match map.entry(name.to_string()) {
                hash_map::Entry::Occupied(entry) => entry.get().clone(),
                hash_map::Entry::Vacant(entry) => {
                    if let Some(v) = f(&name)? {
                        entry.insert(getter(v)).clone()
                    } else {
                        bail!("Unable to evaluate {kind} {name}")
                    }
                }
            };
            match id {
                Some(id) if id != id_from_name => bail!(
                    "Requested {kind} ID {id} and {name}, which resolved to a different ID \
                     ({id_from_name})"
                ),
                _ => Ok(Some(id_from_name.clone())),
            }
        }
    }
}

impl Attributes {
    fn eval(&self, cache: &mut Cache) -> Result<(Permissions, Option<Uid>, Option<Gid>)> {
        let permissions = Permissions::from_mode(u32::from_str_radix(&self.mode, 8)?);

        let uid = get_id(
            &mut cache.uid_map,
            self.uid.map(Uid::from_raw),
            &self.user,
            |name| Ok(User::from_name(&name)?),
            |user| user.uid,
            "user",
        )?;
        let gid = get_id(
            &mut cache.gid_map,
            self.gid.map(Gid::from_raw),
            &self.group,
            |name| Ok(Group::from_name(&name)?),
            |group| group.gid,
            "group",
        )?;

        Ok((permissions, uid, gid))
    }

    fn apply(&self, cache: &mut Cache, p: &Path) -> Result<()> {
        let (permissions, uid, gid) = self.eval(cache)?;
        debug!("Applying chmod {:#04o} to {p:?}", permissions.mode());
        std::fs::set_permissions(p, permissions)?;
        if uid != None || gid != None {
            debug!("Applying chown uid={uid:?} gid={gid:?} to {p:?}");
            nix::unistd::chown(p, uid, gid)?;
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
struct DirEntry {
    attributes: Attributes,
    dirs: Vec<PathBuf>,
}

impl DirEntry {
    fn install_local(&self, cache: &mut Cache, install_dir: &Path) -> Result<()> {
        for dest in &self.dirs {
            let dest = get_install_path(install_dir, dest);
            debug!("Creating directory {dest:?}");
            cache.create_dir(&dest)?;
            self.attributes.apply(cache, &dest)?;
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
struct SymlinkEntry {
    attributes: Attributes,
    destination: PathBuf,
    target: PathBuf,
}

impl SymlinkEntry {
    fn install_local(&self, cache: &mut Cache, install_dir: &Path) -> Result<()> {
        let dest = get_install_path(install_dir, &self.destination);
        cache.create_dir(dest.parent().context("File must have parent")?)?;
        debug!("Creating symlink from {dest:?} to {:?}", self.target);
        std::os::unix::fs::symlink(&self.target, &dest)?;

        // Symlinks don't have a file mode, and chown doesn't work with
        // symlinks (they need lchown to chown the symlink and not the target).
        let (_, uid, gid) = self.attributes.eval(cache)?;
        std::os::unix::fs::lchown(dest, uid.map(Uid::as_raw), gid.map(Gid::as_raw))?;
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
struct FileEntry {
    attributes: Attributes,
    dest_src_map: BTreeMap<PathBuf, String>,
}

impl FileEntry {
    fn install_local(&self, cache: &mut Cache, install_dir: &Path) -> Result<()> {
        for (dest, src) in &self.dest_src_map {
            let dest = get_install_path(install_dir, dest);
            let src = runfiles::rlocation!(cache.runfiles, src);
            cache.create_dir(dest.parent().context("File must have parent")?)?;
            debug!("Copying file from {src:?} to {dest:?}");
            std::fs::copy(&src, &dest)?;
            self.attributes.apply(cache, &dest)?;
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
pub struct Manifest {
    dirs: Vec<DirEntry>,
    files: Vec<FileEntry>,
    symlinks: Vec<SymlinkEntry>,
}

impl Manifest {
    pub fn create(path: &Path) -> Result<Self> {
        Ok(serde_json::from_reader(File::open(path)?)?)
    }

    pub fn install_local(&self, install_dir: &Path) -> Result<()> {
        if nix::unistd::geteuid() != Uid::from_raw(0) {
            bail!("Cannot install package unless running as root (chown only works as root)");
        }

        let mut cache = Cache::create()?;
        for file in &self.files {
            file.install_local(&mut cache, install_dir)?;
        }
        for symlink in &self.symlinks {
            symlink.install_local(&mut cache, install_dir)?;
        }
        // Do the directories last. This way, if we generate a read-only
        // directory, we make sure it has files first.
        for dir in &self.dirs {
            dir.install_local(&mut cache, install_dir)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn gets_correct_attrs() -> Result<()> {
        let mut cache = Cache::create()?;

        let mut assert_root = |attr: Attributes| -> Result<()> {
            let (permissions, uid, gid) = attr.eval(&mut cache)?;
            assert_eq!(permissions, Permissions::from_mode(0o755));
            assert_eq!(uid, Some(Uid::from_raw(0)));
            assert_eq!(gid, Some(Gid::from_raw(0)));
            Ok(())
        };
        assert_root(Attributes {
            mode: "0755".to_string(),
            uid: Some(0),
            gid: Some(0),
            user: None,
            group: None,
        })?;
        assert_root(Attributes {
            mode: "0755".to_string(),
            uid: None,
            gid: None,
            user: Some("root".to_string()),
            group: Some("root".to_string()),
        })?;
        assert_root(Attributes {
            mode: "0755".to_string(),
            uid: Some(0),
            gid: Some(0),
            user: Some("root".to_string()),
            group: Some("root".to_string()),
        })?;
        // Ids don't match.
        assert!(Attributes {
            mode: "0755".to_string(),
            uid: Some(1),
            gid: Some(1),
            user: Some("root".to_string()),
            group: Some("root".to_string()),
        }
        .eval(&mut cache)
        .is_err());
        Ok(())
    }
}
