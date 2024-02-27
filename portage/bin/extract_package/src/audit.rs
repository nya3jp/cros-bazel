// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    fmt::Display,
    fs::DirBuilder,
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use anyhow::{ensure, Context, Result};
use container::{BindMount, ContainerSettings};
use nix::mount::{umount2, MntFlags};
use runfiles::Runfiles;
use tempfile::{NamedTempFile, TempDir};
use vdb::get_vdb_dir;

/// A list of directories to create in the mock environment where we run binary
/// package hooks. This list doesn't need to be exhaustive for the hook check
/// to be accurate, but adding popular directories here will help debugging
/// issues. For example, when a hook accesses `/etc/passwd` while `/etc` is not
/// in this list, the hook check can only report that `/etc` was accessed, not
/// that `/etc/passwd` was accessed, because the kernel doesn't lookup
/// `/etc/passwd` after getting ENOENT for `/etc`.
const MOCK_DIRS: &[&str] = &[
    "bin",
    "build",
    "etc",
    "home",
    "lib",
    "lib32",
    "lib64",
    "media",
    "mnt",
    "opt",
    "packages",
    "root",
    "run",
    "sbin",
    "tmp",
    "tmp/ebuild", // Set to $T
    "usr",
    "usr/bin",
    "usr/include",
    "usr/lib",
    "usr/lib32",
    "usr/lib64",
    "usr/libexec",
    "usr/local",
    "usr/local/bin",
    "usr/local/include",
    "usr/local/lib",
    "usr/local/lib32",
    "usr/local/lib64",
    "usr/local/libexec",
    "usr/local/sbin",
    "usr/local/share",
    "usr/sbin",
    "usr/share",
    "var",
];

struct AuditfuseDaemon {
    mount_dir: PathBuf,
}

impl AuditfuseDaemon {
    pub fn start(audit_file: &Path, orig_dir: &Path, mount_dir: &Path) -> Result<Self> {
        let runfiles = Runfiles::create()?;
        let auditfuse_path =
            runfiles.rlocation("cros/bazel/portage/bin/auditfuse/auditfuse_/auditfuse");

        let status = Command::new(auditfuse_path)
            .arg("--output")
            .arg(audit_file)
            .arg(orig_dir)
            .arg(mount_dir)
            .status()?;

        ensure!(status.success(), "auditfuse failed to start");

        Ok(Self {
            mount_dir: mount_dir.to_path_buf(),
        })
    }
}

impl Drop for AuditfuseDaemon {
    fn drop(&mut self) {
        umount2(&self.mount_dir, MntFlags::MNT_DETACH).expect("unmounting auditfuse");
    }
}

fn drive_binary_package_with_auditfuse(
    cpf: &str,
    root_dir: &Path,
    vdb_dir: &Path,
) -> Result<NamedTempFile> {
    let stage_dir = TempDir::new()?;
    let stage_dir = stage_dir.path();

    let mut builder = DirBuilder::new();
    builder.mode(0o755);
    builder.recursive(true);
    for name in MOCK_DIRS {
        builder.create(stage_dir.join(name))?;
    }

    let audit_file = tempfile::Builder::new()
        .prefix("check_skip_hooks")
        .suffix(".json")
        .tempfile()?;

    let fuse_dir = TempDir::new()?;
    let fuse_dir = fuse_dir.path();

    let _auditfuse_daemon = AuditfuseDaemon::start(audit_file.path(), stage_dir, fuse_dir)
        .context("Failed to start auditfuse")?;

    let mut settings = ContainerSettings::new();
    settings.push_layer(fuse_dir)?;

    let runfiles = Runfiles::create()?;
    settings.push_bind_mount(BindMount {
        source: runfiles.rlocation("files/bash-static"),
        mount_path: PathBuf::from("/bin/bash"),
        rw: false,
    });
    settings.push_bind_mount(BindMount {
        source: runfiles
            .rlocation("cros/bazel/portage/bin/drive_binary_package/drive_binary_package.sh"),
        mount_path: PathBuf::from("/bin/drive_binary_package.sh"),
        rw: false,
    });
    settings.push_bind_mount(BindMount {
        source: vdb_dir.to_path_buf(),
        mount_path: get_vdb_dir(root_dir, cpf),
        rw: false,
    });

    let mut container = settings.prepare()?;
    // Ignore the exit status of drive_binary_package.sh. Hooks may abort due to
    // lack of necessary files for example.
    let _ = container
        .command("/bin/drive_binary_package.sh")
        .arg("-r")
        .arg(root_dir)
        .args(["-d", "/.image"])
        .args(["-t", "/tmp/ebuild"])
        .args(["-p", cpf])
        .arg("-n")
        .arg("-v")
        .args(["setup", "preinst", "postinst"])
        .env("SHELL", "/bin/bash")
        .env("PATH", "/bin")
        // Prevent bash from accessing the locale DB on printing messages.
        .env("LC_ALL", "C")
        .status()?;

    Ok(audit_file)
}

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    strum_macros::EnumString,
    strum_macros::Display,
)]
#[strum(serialize_all = "UPPERCASE")]
pub enum AccessType {
    Lookup,
    Readdir,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AuditEntry {
    pub access_type: AccessType,
    pub path: PathBuf,
}

impl Display for AuditEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.access_type, self.path.display())
    }
}

impl FromStr for AuditEntry {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let (access_type, path) = s.split_once('\t').context("Corrupted audit line")?;
        let access_type: AccessType = access_type.parse()?;
        let path = PathBuf::from(path);
        Ok(Self { access_type, path })
    }
}

fn parse_audit_file(path: &Path) -> Result<Vec<AuditEntry>> {
    let content = std::fs::read_to_string(path)?;
    if content.is_empty() {
        return Ok(Vec::new());
    }
    content
        .strip_suffix('\0')
        .context("Corrupted audit output")?
        .split('\0')
        .map(AuditEntry::from_str)
        .collect::<Result<Vec<_>>>()
}

const LOOKUP_ALLOWLIST: &[&str] = &[
    "/DURABLE_TREE", // Needed internally to mount containers
    "/bin",
    "/dev",
    "/host", // Needed internally to mount containers
    "/proc",
    "/sys",
];

fn filter_audit_entry(entry: &AuditEntry, vdb_root_dir: &Path) -> bool {
    if entry.access_type == AccessType::Lookup {
        // Allow allowlisted files.
        if let Some(path) = entry.path.to_str() {
            if LOOKUP_ALLOWLIST.contains(&path) {
                return false;
            }
        }

        // Allow looking up the VDB directory of the current package and its ancestors.
        if vdb_root_dir.ancestors().any(|p| entry.path == p) {
            return false;
        }
    }

    // Allow inspecting the VDB directory of the current package.
    if entry.path.starts_with(vdb_root_dir) {
        return false;
    }

    true
}

pub fn audit_hooks(cpf: &str, root_dir: &Path, vdb_dir: &Path) -> Result<Vec<AuditEntry>> {
    let audit_file = drive_binary_package_with_auditfuse(cpf, root_dir, vdb_dir)?;

    let entries = parse_audit_file(audit_file.path())?;

    let vdb_root_dir = get_vdb_dir(root_dir, cpf);

    let entries = entries
        .into_iter()
        .filter(|e| filter_audit_entry(e, &vdb_root_dir))
        .collect();

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    // Run unit tests in a mount namespace to use durable trees.
    #[used]
    #[link_section = ".init_array"]
    static _CTOR: extern "C" fn() = ::testutil::ctor_enter_mount_namespace;

    /// A short-cut function to create [`AuditEntry`].
    fn lookup(path: &str) -> AuditEntry {
        AuditEntry {
            access_type: AccessType::Lookup,
            path: PathBuf::from(path),
        }
    }

    /// A short-cut function to create [`AuditEntry`].
    fn readdir(path: &str) -> AuditEntry {
        AuditEntry {
            access_type: AccessType::Readdir,
            path: PathBuf::from(path),
        }
    }

    /// Runs [`audit_hooks`] for unit testing.
    fn run_audit_hooks(extra_env: &str) -> Result<Vec<AuditEntry>> {
        const BASE_ENV: &str = r#"
EAPI=7
SLOT=0
KEYWORDS="*"
CATEGORY=virtual
PF=test-1
"#;

        let vdb_dir = TempDir::new()?;
        let vdb_dir = vdb_dir.path();

        std::fs::write(
            vdb_dir.join("environment.raw"),
            format!("{}{}", BASE_ENV, extra_env),
        )?;
        std::fs::write(vdb_dir.join("repository"), "chromiumos")?;

        audit_hooks("virtual/test-1", Path::new("/"), vdb_dir)
    }

    #[test]
    fn test_audit_hooks_undefined() -> Result<()> {
        let entries = run_audit_hooks("")?;
        assert_eq!(entries, Vec::new());
        Ok(())
    }

    #[test]
    fn test_audit_hooks_empty() -> Result<()> {
        let entries = run_audit_hooks(
            r#"
pkg_setup() {
    :
}

pkg_preinst() {
    :
}

pkg_postinst() {
    :
}
"#,
        )?;
        assert_eq!(entries, Vec::new());
        Ok(())
    }

    #[test]
    fn test_audit_hooks_lookup() -> Result<()> {
        let entries = run_audit_hooks(
            r#"
pkg_setup() {
    : < /etc/a
}

pkg_preinst() {
    : < /etc/b
}

pkg_postinst() {
    : < /etc/c
}
"#,
        )?;
        assert_eq!(
            entries,
            vec![
                lookup("/etc"),
                lookup("/etc/a"),
                lookup("/etc/b"),
                lookup("/etc/c"),
            ]
        );
        Ok(())
    }

    #[test]
    fn test_audit_hooks_readdir() -> Result<()> {
        let entries = run_audit_hooks(
            r#"
pkg_setup() {
    echo /mnt/*
}

pkg_preinst() {
    echo /opt/*
}

pkg_postinst() {
    echo /run/*
}
"#,
        )?;
        assert_eq!(
            entries,
            vec![
                lookup("/mnt"),
                readdir("/mnt"),
                lookup("/opt"),
                readdir("/opt"),
                lookup("/run"),
                readdir("/run"),
            ]
        );
        Ok(())
    }

    #[test]
    fn test_audit_hooks_conditional() -> Result<()> {
        // Exercise the common pattern to guard hooks with $MERGE_TYPE.
        let entries = run_audit_hooks(
            r#"
pkg_setup() {
    if [[ "${MERGE_TYPE}" == binary ]]; then
        return
    fi
    : < /etc/a
}

pkg_preinst() {
    [[ "${MERGE_TYPE}" == binary ]] && return
    : < /etc/b
}

pkg_postinst() {
    [[ "${MERGE_TYPE}" == ebuild ]] || return
    : < /etc/c
}
"#,
        )?;
        assert_eq!(entries, Vec::new());
        Ok(())
    }

    #[test]
    fn test_audit_hooks_builtins() -> Result<()> {
        // It's allowed to call bash builtins.
        let entries = run_audit_hooks(
            r#"
call_builtins() {
    export FOO=bar
    compgen -A function pkg_
    declare -F foobar
    true
    false
}

pkg_setup() {
    call_builtins
}

pkg_preinst() {
    call_builtins
}

pkg_postinst() {
    call_builtins
}
"#,
        )?;
        assert_eq!(entries, Vec::new());
        Ok(())
    }

    #[test]
    fn test_audit_hooks_allow_vdb() -> Result<()> {
        // Reading from the VDB directory of the current package is allowed.
        let entries = run_audit_hooks(
            r#"
ensure_vdb_access() {
    local repo=$(< "${ROOT}/var/db/pkg/${CATEGORY}/${PF}/repository")
    if [[ "${repo}" != chromiumos ]]; then
        : < /fail
    fi
}

pkg_setup() {
    ensure_vdb_access
}

pkg_preinst() {
    ensure_vdb_access
}

pkg_postinst() {
    ensure_vdb_access
}
"#,
        )?;
        assert_eq!(entries, Vec::new());
        Ok(())
    }

    #[test]
    fn test_audit_hooks_allow_die() -> Result<()> {
        // Some hooks may die due to lack of filesystem access, but it should
        // not make extract_package fail.
        let entries = run_audit_hooks(
            r#"
pkg_setup() {
    if [[ ! -e /etc/foobar.conf ]]; then
        die "foobar.conf is not found"
    fi
}
"#,
        )?;
        assert_eq!(entries, vec![lookup("/etc"), lookup("/etc/foobar.conf"),]);
        Ok(())
    }
}
