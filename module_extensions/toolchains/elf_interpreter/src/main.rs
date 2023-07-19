// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod elf;
mod helpers;
mod real_main;

use std::convert::TryInto;

// This binary:
// * Needs access to std (in order to use std::File, std::Path, etc.)
//   * We could implement this ourselves with syscalls if we really needed to, but that would make
//     this far less safe.
// * Needs access to libc (the library, not the crate).
//   * For file-support.
//   * For various helper methods (eg. memcpy, memset) used for memory allocation
// * Cannot depend on any shared libraries
//   * We need to link against musl instead of glibc, since glibc doesn't support static linking.
// * Cannot depend on anything that the elf interpreter is expected to set up for us.
//   * This program runs before the system-installed elf interpreter runs.
//   * std::rt::lang_start fails with an error "assertion failed: thread_info.is_none()".

// For the reasons above, the crate cannot be no_std, since we need std functions, but it can't be
// std either, since std::rt::lang_start doesn't work. Instead, we have to build a std crate, then
// make main invoke __wrap_main rather than libc's main.

// This function never gets called, but rust requires a main.
pub fn main() {}

// See line 95 in https://git.musl-libc.org/cgit/musl/tree/src/env/__libc_start_main.c?id=718f363bc2067b6487900eddc9180c84e7739f80#n95
// Note that this does not wrap the above main, which is mangled, but instead wraps libc's main.
#[no_mangle]
pub(crate) extern "C" fn __wrap_main(
    argc: core::ffi::c_int,
    argv: *const *const core::ffi::c_char,
    _envp: *const *const core::ffi::c_char,
) -> core::ffi::c_int {
    // Turn pointers into slices so that we can have rust's safety guarantees.
    let args = unsafe { core::slice::from_raw_parts(argv, argc.try_into().unwrap()) };

    match real_main::real_main(args) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Error during ELF loader: {e:?}");
            // Exit with a non-reserved status code that a user is unlikely to
            // do themselves.
            254
        }
    }
}
