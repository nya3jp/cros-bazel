#!/bin/bash -ue

# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

if [[ ! -e /etc/cros_chroot_version ]]; then
  echo "Cannot build ':installed' targets outside the cros SDK chroot."
  exit 1
fi

BINPKG="$1"
shift
DEST="$1"
shift
EMERGE_CMD="$1"
shift
CHECKSUM="$1"
shift

sudo mkdir -p "$(dirname "${DEST}")"

# Ideally we'd just use a symlink here for performance reasons. However, if you
# create a symlink for package foo, then foo emerges just fine, but any package
# that rdepends on it will fail to detect it during emerge.
sudo cp "${BINPKG}" "${DEST}"
sudo chmod 644 "${DEST}"

# Calculate the checksums in parallel with emerging.
(cat "${BINPKG}" "$@" | sha256sum > "${CHECKSUM}" ) &

# Note: Despite only installing a single package at a time, this operates in
# parallel because emerge doesn't take a lock and allows you to install multiple
# packages at the same time.
# Since we divide this up into one action per package, bazel can automatically
# handle the installation order so that a package can only install once the deps
# are installed.
${EMERGE_CMD}

wait
