// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

/// Enters a new unprivileged mount/user namespace where the process has
/// privilege to mount file systems.
///
/// Register this function to the .init_array section if you want your tests to be
/// run in a mount namespace.
///
/// We need this ugly hack because unshare(CLONE_NEWUSER) fails if the current
/// process has multiple threads, whereas the standard test harness spawns
/// threads before running tests.
///
/// # Example
///
/// ```
/// use testutil::ctor_enter_mount_namespace;
///
/// #[used]
/// #[link_section = ".init_array"]
/// static _CTOR: extern "C" fn() = ctor_enter_mount_namespace;
/// ```
pub extern "C" fn ctor_enter_mount_namespace() {
    // WARNING: You MUST NOT use the standard library (std) in this function
    // since it is called before std is initialized!
    // Calling libc functions is almost only things you can do here. Good luck!
    unsafe {
        let uid = libc::getuid();
        let gid = libc::getgid();

        if libc::unshare(libc::CLONE_NEWUSER | libc::CLONE_NEWNS) < 0 {
            panic!("unshare failed");
        }

        unsafe fn write_to_file(path_cstr: &str, data: &str) {
            let file = libc::fopen(
                path_cstr.as_ptr() as *const libc::c_char,
                "w\0".as_ptr() as *const libc::c_char,
            );
            assert!(!file.is_null(), "fopen failed for {}", path_cstr);
            if libc::fwrite(data.as_ptr() as *const libc::c_void, data.len(), 1, file) != 1 {
                panic!("fwrite failed for {}", path_cstr);
            }
            if libc::fclose(file) < 0 {
                panic!("fclose failed for {}", path_cstr);
            }
        }

        write_to_file("/proc/self/setgroups\0", "deny");
        write_to_file("/proc/self/uid_map\0", &format!("0 {} 1\n", uid));
        write_to_file("/proc/self/gid_map\0", &format!("0 {} 1\n", gid));
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};
    use nix::mount::{MntFlags, MsFlags};

    use super::*;

    #[used]
    #[link_section = ".init_array"]
    static _CTOR: extern "C" fn() = ctor_enter_mount_namespace;

    #[test]
    fn test_mount() -> Result<()> {
        // Verify that we can mount tmpfs.
        nix::mount::mount(Some(""), "/", Some("tmpfs"), MsFlags::empty(), Some(""))
            .context("mount failed")?;
        nix::mount::umount2("/", MntFlags::MNT_DETACH).context("umount failed")?;
        Ok(())
    }
}
