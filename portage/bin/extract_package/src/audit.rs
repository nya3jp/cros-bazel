// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::BTreeSet,
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
            .arg("--verbose")
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
    lookup_allowlist: &BTreeSet<&Path>,
) -> Result<NamedTempFile> {
    let stage_dir = TempDir::new()?;
    let stage_dir = stage_dir.path();

    let mut builder = DirBuilder::new();
    builder.mode(0o755);
    builder.recursive(true);

    for name in MOCK_DIRS {
        builder.create(stage_dir.join(name))?;
    }

    // Directories listed in the lookup allowlist must exist. See `filter_audit_entry`.
    for rel_path in lookup_allowlist {
        builder.create(
            stage_dir.join(
                rel_path
                    .strip_prefix("/")
                    .with_context(|| format!("Must be an absolute path: {}", rel_path.display()))?,
            ),
        )?;
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

/// A static list of directories allowlisted for lookup.
const LOOKUP_ALLOWED_DIRS: &[&str] = &[
    // Directories listed here must satisfy certain conditions to avoid false positives.
    // See comments in [`filter_audit_entry`] for details.
    "/bin", "/dev",
    // /host is used for pivot_root during the setup of a container, but is empty afterwards.
    "/host", "/proc", "/sys",
];

fn filter_audit_entry(
    entry: &AuditEntry,
    vdb_root_dir: &Path,
    lookup_allowlist: &BTreeSet<&Path>,
) -> bool {
    // It is known to safe to ignore an entry in the following cases:
    //
    // 1. Ignoring a lookup of a directory or a file that universally exists on a functioning system
    //    (e.g. /bin). In the case of a directory, it must exist at the time of auditing; otherwise,
    //    ignoring a lookup may result in false negatives. For example, if a lookup of /bin is
    //    ignored while the directory does not exist, stat("/bin/ls") incorrectly leaves no audit
    //    entry because the kernel does not look up /bin/ls after failing to look up /bin.
    // 2. Ignoring a lookup or readdir of all descendant directories/files under a directory that
    //    is known to exclusively belong to the current package (e.g. VDB directory).
    if entry.access_type == AccessType::Lookup && lookup_allowlist.contains(entry.path.as_path()) {
        return false;
    }
    if entry.path.starts_with(vdb_root_dir) {
        return false;
    }
    // HACK: /DURABLE_TREE is a regular file looked up internally by the durabletree crate on
    // setting up a container.
    if entry.path == Path::new("/DURABLE_TREE") {
        return false;
    }
    true
}

pub fn audit_hooks(cpf: &str, root_dir: &Path, vdb_dir: &Path) -> Result<Vec<AuditEntry>> {
    let vdb_root_dir = get_vdb_dir(root_dir, cpf);
    let lookup_allowlist: BTreeSet<&Path> = LOOKUP_ALLOWED_DIRS
        .iter()
        .map(Path::new)
        .chain(vdb_root_dir.ancestors())
        .collect();

    let audit_file =
        drive_binary_package_with_auditfuse(cpf, root_dir, vdb_dir, &lookup_allowlist)?;

    let entries = parse_audit_file(audit_file.path())?;

    let entries = entries
        .into_iter()
        .filter(|e| filter_audit_entry(e, &vdb_root_dir, &lookup_allowlist))
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

        audit_hooks("virtual/test-1", Path::new("/build/amd64-generic"), vdb_dir)
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
    fn test_audit_hooks_allowlist() -> Result<()> {
        let entries = run_audit_hooks(
            r#"
do_lookup() {
    # These entries are ignored.
    : < /DURABLE_TREE
    : < /bin
    : < /dev
    : < /host
    : < /proc
    : < /sys
    : < ${ROOT}
    : < ${ROOT}/var
    : < ${ROOT}/var/db
    : < ${ROOT}/var/db/pkg
    : < ${ROOT}/var/db/pkg/virtual
    : < ${ROOT}/var/db/pkg/virtual/test-1
    : < ${ROOT}/var/db/pkg/virtual/test-1/foo
    : < ${ROOT}/var/db/pkg/virtual/test-1/foo/bar

    # These entries are detected.
    : < /usr
    : < /var/cache
    : < ${ROOT}/usr
    : < ${ROOT}/var/cache
}

pkg_setup() {
    do_lookup
}

pkg_preinst() {
    do_lookup
}

pkg_postinst() {
    do_lookup
}
"#,
        )?;
        assert_eq!(
            entries,
            vec![
                lookup("/usr"),
                lookup("/var"),
                lookup("/var/cache"),
                lookup("/build/amd64-generic/usr"),
                // /build/amd64-generic/var is ignored because it's an ancestor of the VDB dir.
                lookup("/build/amd64-generic/var/cache"),
            ],
        );
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
        assert_eq!(entries, vec![lookup("/etc"), lookup("/etc/foobar.conf")]);
        Ok(())
    }
}
