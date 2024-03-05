#!/bin/bash -e

# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

OUT_DIR="$1"
TARBALL="$2"

TAR_OPTS=()
if type pixz &> /dev/null; then
  TAR_OPTS+=("-Ipixz")
fi

tar -xf "${TARBALL}" -C "${OUT_DIR}" "${TAR_OPTS[@]}"

DST="usr/x86_64-cros-linux-gnu/"
# Use --link-dest to generate hardlinks instead of copying files.
rsync --archive --link-dest="${DST}" "${DST}" .
