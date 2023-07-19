// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::helpers::{slice_from_ptrs, PAGE_SIZE};
use anyhow::{Context, Result};
use elfloader::ElfLoaderErr::ElfParser;
use elfloader::{ElfLoader, ElfLoaderErr, Flags, LoadableHeaders, RelocationEntry, VAddr};

static PAGE_MASK: u64 = !(PAGE_SIZE - 1);

pub(crate) struct Loader<'a> {
    pub(crate) vbase: u64,
    pub(crate) blob: &'a [u8],
}

impl Loader<'_> {
    pub fn relocate_address(&self, addr: u64) -> u64 {
        self.vbase + addr
    }

    fn allocate_internal(&mut self, load_headers: LoadableHeaders) -> Result<()> {
        for header in load_headers {
            let flags = header.flags();
            let addr = self.relocate_address(header.virtual_addr());

            // Page-align the start and end of the sections.
            let mmap_slice = unsafe {
                slice_from_ptrs(
                    (addr & PAGE_MASK) as *mut u8,
                    ((addr + header.mem_size() + (PAGE_SIZE - 1)) & PAGE_MASK) as *mut u8,
                )
            };

            // We could use a non-anonymous mmap and thus avoid the
            // memcpy / mprotect, but this ensures we don't leak a file handle
            // to ld.so.
            crate::helpers::mmap(
                mmap_slice,
                // Technically, we only need write, but I've previously had
                // issues where you're unable to write without the read
                // permission.
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_FIXED | libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
                // Anonymous requires you to set the file descriptor to -1
                // on some systems.
                -1,
                0,
            )?;

            let offset: usize = header
                .offset()
                .try_into()
                .map_err(anyhow::Error::msg)
                .context("offset too big")?;
            let file_size: usize = header
                .file_size()
                .try_into()
                .map_err(anyhow::Error::msg)
                .context("file size too big")?;
            unsafe { core::slice::from_raw_parts_mut(addr as *mut u8, file_size) }
                .copy_from_slice(&self.blob[offset..][..file_size]);

            let get_mmap_flag = |enabled, mask| if enabled { mask } else { 0 };
            crate::helpers::mprotect(
                mmap_slice,
                get_mmap_flag(flags.is_read(), libc::PROT_READ)
                    | get_mmap_flag(flags.is_write(), libc::PROT_WRITE)
                    | get_mmap_flag(flags.is_execute(), libc::PROT_EXEC),
            )?;
        }
        Ok(())
    }
}

// Our logic is a bit wierd, because this crate expects you to be doing things
// from the kernel, where you can memcpy to physical addresses in the allocate
// function and then relocate the virtual address space later in the relocate
// and load functions respectively.
impl ElfLoader for Loader<'_> {
    fn allocate(&mut self, load_headers: LoadableHeaders) -> Result<(), ElfLoaderErr> {
        // Unfortunately the trait forces us to return useless information.
        self.allocate_internal(load_headers).map_err(|e| {
            eprintln!("Got error {e:?}");
            ElfParser {
                source: "Error while loading elf file: see stderr for details",
            }
        })
    }

    fn relocate(&mut self, _entry: RelocationEntry) -> Result<(), ElfLoaderErr> {
        Ok(())
    }

    fn load(&mut self, _flags: Flags, _base: VAddr, _region: &[u8]) -> Result<(), ElfLoaderErr> {
        Ok(())
    }

    fn tls(
        &mut self,
        _tdata_start: VAddr,
        _tdata_length: u64,
        _total_size: u64,
        _align: u64,
    ) -> Result<(), ElfLoaderErr> {
        // This should never be the case for ld-linux.so.
        unreachable!()
    }
}
