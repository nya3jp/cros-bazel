#!/bin/bash -ex
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

export ROOT="/${BOARD:+build/${BOARD}/}"
export SYSROOT="${ROOT}"
export PORTAGE_CONFIGROOT="${ROOT}"

# Install libc to sysroot.
#
# There is no reason we can't compile glibc for the target. This is here just
# to support the legacy toolchain setup processes.
#
# Logic borrowed from chromite/lib/toolchain.py.
rm -rf /tmp/libc
mkdir -p /tmp/libc

tar -I "zstd -f" -x -f "/mnt/host/.sdk_install_glibc/glibc.tbz2" -C /tmp/libc
mkdir -p "/build/${BOARD}" "/build/${BOARD}/usr/lib/debug"
rsync --archive --hard-links /tmp/libc/usr/*-cros-linux-gnu/ "/build/${BOARD}/"
# TODO(b/278728702): Once our cross-$CTARGE/glibc package includes debug
# symbols remove this check.
if [[ -d /tmp/libc/usr/lib/debug/usr ]]; then
  rsync --archive --hard-links /tmp/libc/usr/lib/debug/usr/*-cros-linux-gnu/ \
    "/build/${BOARD}/usr/lib/debug/"
fi
