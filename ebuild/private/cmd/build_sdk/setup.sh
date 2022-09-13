#!/bin/bash -ex
# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# HACK: Print all outputs to stderr to avoid shuffled logs in Bazel output.
exec >&2

export LANG=en_US.UTF-8
export PORTAGE_USERNAME=root
export PORTAGE_GRPNAME=root
export RESTRICT="fetch"
export FEATURES="digest -sandbox -usersandbox"  # TODO: turn on sandbox

for i in /stage/tarballs/*; do
  tar -xv -f "${i}" -C /
done

locale-gen --jobs 1

# TODO: Consider using fakeroot-like approach to emulate file permissions.
sed -i -e '/dir_mode_map = {/,/}/s/False/True/' /usr/lib/python3.6/site-packages/portage/package/ebuild/config.py

# HACK: Do not use namespaces in ebuild(1).
# TODO: Find a better way.
sed -i "/keywords\['unshare/d" /usr/lib/python3.6/site-packages/portage/package/ebuild/doebuild.py

read -ra atoms <<<"${INSTALL_ATOMS_HOST}"
if (( ${#atoms[@]} )); then
  # TODO: emerge is too slow! Find a way to speed up.
  time emerge --oneshot --usepkgonly --nodeps --jobs=16 "${atoms[@]}"
fi

read -ra atoms <<<"${INSTALL_ATOMS_TARGET}"
if (( ${#atoms[@]} )); then
  # TODO: emerge is too slow! Find a way to speed up.
  time ROOT="/build/${BOARD}/" SYSROOT="/build/${BOARD}/" PORTAGE_CONFIGROOT="/build/${BOARD}/" emerge --oneshot --usepkgonly --nodeps --jobs=16 "${atoms[@]}"
fi

# Install libc to sysroot.
# Logic borrowed from chromite/lib/toolchain.py.
# TODO: Stop hard-coding aarch64-cros-linux-gnu.
rm -rf /tmp/libc
mkdir -p /tmp/libc
tar -I "zstd -f" -x -f "/var/lib/portage/pkgs/cross-aarch64-cros-linux-gnu/glibc-"*.tbz2 -C /tmp/libc
mkdir -p "/build/${BOARD}" "/build/${BOARD}/usr/lib/debug"
rsync --archive --hard-links "/tmp/libc/usr/aarch64-cros-linux-gnu/" "/build/${BOARD}/"
rsync --archive --hard-links "/tmp/libc/usr/lib/debug/usr/aarch64-cros-linux-gnu/" "/build/${BOARD}/usr/lib/debug/"
