// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};

pub(crate) static PAGE_SIZE: u64 = 4096;

#[derive(Eq, PartialEq)]
pub(crate) struct AuxEntry {
    pub tag: u64,
    pub value: u64,
}

/// Precondition: start < end
pub(crate) unsafe fn ptr_distance<T>(start: *const T, end: *const T) -> usize {
    let distance = end.offset_from(start);
    assert!(distance >= 0);
    distance.try_into().unwrap_unchecked()
}

pub(crate) unsafe fn slice_from_ptrs<'a, T>(start: *mut T, end: *mut T) -> &'a mut [T] {
    unsafe { core::slice::from_raw_parts_mut(start, ptr_distance(start, end)) }
}

/// This returns a slice up to, and including, the first element failing the
/// condition.
pub(crate) unsafe fn take_from_ptr_while<T: Sized, F>(
    start: *mut T,
    condition: F,
) -> &'static mut [T]
where
    F: Fn(&T) -> bool,
{
    let mut end = start;
    while condition(&*end) {
        end = unsafe { end.add(1) };
    }
    unsafe { slice_from_ptrs(start, end.add(1)) }
}

pub(crate) fn mmap<T: Sized>(
    slice: &mut [T],
    prot: i32,
    flags: i32,
    fd: i32,
    offset: i64,
) -> Result<&mut [T]> {
    let mmap_result = unsafe {
        libc::mmap(
            slice.as_mut_ptr().cast::<libc::c_void>(),
            slice.len() * std::mem::size_of::<T>(),
            prot,
            flags,
            fd,
            offset,
        )
    };
    if mmap_result == libc::MAP_FAILED {
        Err(std::io::Error::last_os_error()).context("Failed to mmap segment")
    } else {
        Ok(unsafe { core::slice::from_raw_parts_mut(mmap_result.cast::<T>(), slice.len()) })
    }
}

pub(crate) fn mprotect<T: Sized>(slice: &mut [T], flags: i32) -> Result<()> {
    let mprotect_result = unsafe {
        libc::mprotect(
            slice.as_mut_ptr().cast::<libc::c_void>(),
            slice.len() * std::mem::size_of::<T>(),
            flags,
        )
    };
    if mprotect_result != 0 {
        Err(std::io::Error::last_os_error()).context("Failed to mprotect segment")
    } else {
        Ok(())
    }
}
