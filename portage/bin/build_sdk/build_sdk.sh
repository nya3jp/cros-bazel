#!/bin/bash -x
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

set -eu -o pipefail

export ROOT="/${BOARD:+build/${BOARD}/}"

# Create symlinks to do the same thing as src/scripts/build_sdk_board.
mkdir -p "${ROOT}/mnt/host"
ln -s /mnt/host/source/src/chromium/depot_tools "${ROOT}/mnt/host/depot_tools"

# clean_layer is a bit too aggressive. Let's recreate the cleared out
# directories.
mkdir -p "${ROOT}/var/log/portage/elog"
mkdir -p "${ROOT}/var/log/sandbox"
mkdir -p "${ROOT}/var/tmp"
mkdir -p "${ROOT}/var/cache"
mkdir -p "${ROOT}/tmp"
chmod a+w,+t "${ROOT}/tmp"
chmod a+w,+t "${ROOT}/var/tmp"
chmod 0770 "${ROOT}/var/log/sandbox"
fakeroot chown 0:250 "${ROOT}/var/log/sandbox"

fakeroot env-update

# Needed to tell chromite's cros_build_lib that we are running inside the
# SDK. We don't use a real version number since there is no such thing in the
# bazel world.
echo bazel > "${ROOT}/etc/cros_chroot_version"

# We don't want tar to change the permissions on the root directory when
# we extract it.
chmod 755 "${ROOT}"

# Copy the sysroot into the output directory so we can create a new durable
# tree. We don't invoke fakeroot since we only need to copy the xattrs to
# preserve ownership.
time tar \
  --format gnu \
  --sort name \
  --mtime "1970-1-1 00:00Z" \
  --numeric-owner \
  --create \
  --directory "${ROOT}" \
  --exclude "./tmp/*" \
  --exclude "./var/cache/*" \
  --exclude "./packages" \
  --exclude "./build" \
  --exclude "./usr/share/doc/*" \
  --exclude "./usr/share/man/*" \
  --exclude="./etc/make.conf" \
  --exclude="./etc/make.conf.*" \
  --exclude="./etc/portage" \
  . | \
  tar -x -C "/mnt/host/.build_sdk/output"
