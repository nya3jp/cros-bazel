// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Result};
use core::ffi::c_char;
use std::path::Path;

// This struct is how we will send data to the real elf interpreter.
#[repr(C)]
pub(crate) struct CrosMetadata {
    sysroot: *const c_char,
}

// We have three places we could place the data we want to pass to the real elf
// interpreter:
// * Stack - This is undefined behaviour, since it's reused and overridden.
// * Heap - this appears to sometimes work, and sometimes not. I have a
//     suspicion that the elf interpreter is trying to use the same heap as us
//     and sometimes overwrites the memory.
// * Data - this consistently works, because this stays loaded.
const STATIC_ALLOCATOR_N_PAGES: usize = 1;
const STATIC_ALLOCATOR_MAX_SIZE: usize =
    STATIC_ALLOCATOR_N_PAGES * 4096 - std::mem::size_of::<usize>();

#[repr(align(4096))]
struct StaticAllocator {
    upto: usize,
    bytes: [u8; STATIC_ALLOCATOR_MAX_SIZE],
}

impl StaticAllocator {
    pub const fn new() -> Self {
        Self {
            upto: 0,
            bytes: [0; STATIC_ALLOCATOR_MAX_SIZE],
        }
    }

    pub fn allocate<T: Sized>(&mut self) -> Result<&mut T> {
        let bytes = self.allocate_bytes(std::mem::size_of::<T>())?;
        let ptr: *mut T = bytes.as_mut_ptr().cast::<T>();
        Ok(unsafe { &mut *ptr })
    }

    pub fn allocate_bytes(&mut self, n: usize) -> Result<&mut [u8]> {
        if self.upto + n > STATIC_ALLOCATOR_MAX_SIZE {
            bail!("Ran out of space in static allocator for elf interpreter");
        }
        let bytes: &mut [u8] = &mut self.bytes[self.upto..][..n];
        self.upto += n;
        Ok(bytes)
    }
}

static mut STATIC_ALLOCATOR: StaticAllocator = StaticAllocator::new();

fn generate_static_c_string(s: &str) -> Result<*const c_char> {
    let s = std::ffi::CString::new(s)?;
    let src = s.as_bytes_with_nul();
    // Accessing static muts is considered unsafe for thread safety.
    let dst: &mut [u8] = unsafe { STATIC_ALLOCATOR.allocate_bytes(src.len())? };
    dst.copy_from_slice(src);
    Ok(dst.as_ptr().cast::<c_char>())
}

pub(crate) fn serialize(sysroot: &Path) -> Result<*const CrosMetadata> {
    // Accessing static muts is considered unsafe for thread safety.
    let mut serialized: &mut CrosMetadata = unsafe { STATIC_ALLOCATOR.allocate()? };
    serialized.sysroot = generate_static_c_string(&sysroot.to_string_lossy().to_string())?;
    Ok(&*serialized)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEMO_PATH: &str = "/foo";

    #[test]
    fn test_allocator_limits() -> Result<()> {
        let mut allocator = StaticAllocator::new();
        let first = allocator.allocate_bytes(STATIC_ALLOCATOR_MAX_SIZE - 5)?;
        assert_eq!(first.len(), STATIC_ALLOCATOR_MAX_SIZE - 5);
        let first_end = first.as_ptr_range().end;

        let second = allocator.allocate_bytes(5)?;
        assert_eq!(second.len(), 5);
        assert_eq!(first_end, second.as_ptr());
        allocator
            .allocate_bytes(1)
            .expect_err("Allocator should have run out of space");

        Ok(())
    }

    #[test]
    fn test_serialization() -> Result<()> {
        let serialized: &CrosMetadata = unsafe { &*serialize(&Path::new(DEMO_PATH))? };
        assert_eq!(
            unsafe { std::ffi::CStr::from_ptr(serialized.sysroot) }.to_bytes(),
            std::ffi::CString::new(DEMO_PATH)?.to_bytes()
        );

        Ok(())
    }
}
