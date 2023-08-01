// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::helpers::{AuxEntry, PAGE_SIZE};
use anyhow::{anyhow, bail, Context, Result};
use core::ffi::{c_char, CStr};

pub(crate) fn real_main(args: &[*const c_char], aux: &mut [AuxEntry]) -> Result<()> {
    // AT_EXECFN contains "A pointer to a string containing the pathname used
    // to execute the program."
    let exec_path = aux
        .iter()
        .find(|entry| entry.tag == libc::AT_EXECFN)
        .context("Must have an execfn entry")?
        .value as *const c_char;
    let exec_path = unsafe { CStr::from_ptr(exec_path) }.to_str()?;
    let r = runfiles::Runfiles::create_with_custom_binary_path(exec_path)?;

    let path = r.rlocation("toolchain_sdk/lib64/ld-linux-x86-64.so.2");
    // We can't look for the sysroot directly since rlocation doesn't support
    // searching for directories.
    let sysroot = path.parent().unwrap().parent().unwrap();
    let binary_blob =
        std::fs::read(&path).with_context(|| format!("Unable to read interpreter at {path:?}"))?;
    let binary = elfloader::ElfBinary::new(binary_blob.as_slice())
        .map_err(|e| anyhow!("Unable to parse interpreter as an elf binary: {e:?}"))?;
    let mut loader = crate::elf::Loader {
        // This works fine for any PIE / PIC binaries but may fail for binaries
        // that don't support ASLR, happen to use this address, and are
        // dynamically linked.
        // Since we always build with PIC, this should be ok for now.
        // This offset was chosen arbitrarily.
        vbase: PAGE_SIZE * 0x1000000,
        blob: &binary_blob,
    };
    binary
        .load(&mut loader)
        .map_err(|e| anyhow!("Unable to load the interpreter: {e:?}"))?;

    let entry = loader.relocate_address(binary.entry_point());

    // We need to pass information to the real elf loader.
    // Ideally we'd just add a new entry to args / env / aux.
    // However, we need to take special care not to resize the stack,
    // as if you do so, everything blows up.
    // To solve this problem, we just write to the value of the AT_NULL entry,
    // which nobody cares if you change.
    let at_null_entry = aux.last_mut().unwrap();
    if at_null_entry.value != 0 {
        bail!("Final entry of auxv (AT_NULL) has non-zero value")
    }
    at_null_entry.value = crate::serialize::serialize(&sysroot)? as u64;

    // rsp needs to point to argc, which is always 8 bytes before argv.
    // See https://articles.manugarg.com/aboutelfauxiliaryvectors.html
    let rsp = unsafe { args.as_ptr().sub(1) };

    unsafe {
        core::arch::asm!(
        // Rust refuses to let you assign to the stack pointer, so we assign
        // to tmp and copy that to the stack pointer.
        "mov rsp, {tmp}",
        // We need to store the location to jump to somewhere.
        // Choose rdi because that's the first register modified by the interp.
        "mov rdi, {entry}",
        // Zero out the registers to ensure consistency with the regular
        // entrypoint.
        "mov rax, 0",
        "mov rbx, 0",
        "mov rcx, 0",
        "mov rdx, 0",
        "mov rsi, 0",
        "mov rbp, 0",
        "mov r8, 0",
        "mov r9, 0",
        "mov r10, 0",
        "mov r11, 0",
        "mov r12, 0",
        "mov r13, 0",
        "mov r14, 0",
        "mov r15, 0",
        "jmp rdi",
        entry = in(reg) entry,
        tmp = in(reg) rsp,
        options(noreturn),
        )
    }
}
