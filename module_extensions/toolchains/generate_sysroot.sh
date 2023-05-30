#!/bin/bash -e

# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

OUT_DIR="$1"
CLANG_SELECTOR_SRC="$2"

# By making this argument optional, this allows us to easily experiment without
# actually running the most expensive part of the repo rule.
if [ $# -gt 2 ]; then
  TARBALL="$3"
  tar -xf "${TARBALL}" -C "${OUT_DIR}"

  # TODO: Make this fully hermetic.
  # This is currently fully hermetic under a few conditions:
  # * exec machine is the host machine (so not RBE)
  # * No two cros checkouts with distinct sysroots.
  # * Not executing inside a chroot / user namespace or similar.
  SYSROOT_SYMLINK="/tmp/cros_bazel_host_sysroot"
  rm -f "${SYSROOT_SYMLINK}"
  ln -s "${OUT_DIR}" "${SYSROOT_SYMLINK}"

fi

DST="usr/x86_64-cros-linux-gnu/"
# Use --link-dest to generate hardlinks instead of copying files.
rsync --archive --link-dest="${DST}" "${DST}" .

# Args taken from running
# "bazel build -s -c opt //bazel/examples/cc:hello_world"
usr/bin/clang++ \
  -Wno-builtin-macro-redefined \
  -D__DATE__=redacted \
  -D__TIMESTAMP__=redacted \
  -D__TIME__=redacted \
  -U_FORTIFY_SOURCE \
  -D_FORTIFY_SOURCE=1 \
  -fstack-protector \
  -Wall \
  -Wunused-but-set-parameter \
  -Wno-free-nonheap-object \
  -no-canonical-prefixes \
  --sysroot . \
  -O3 \
  -g0 \
  -ffunction-sections \
  -fdata-sections \
  -DNDEBUG \
  -std=c++17 \
  -static \
  "${CLANG_SELECTOR_SRC}" \
  -o usr/bin/clang_selector
