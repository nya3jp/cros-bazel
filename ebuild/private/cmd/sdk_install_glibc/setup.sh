#!/bin/bash -ex
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

: "${BOARD?}"

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
rsync --archive --hard-links /tmp/libc/usr/lib/debug/usr/*-cros-linux-gnu/ \
  "/build/${BOARD}/usr/lib/debug/"
