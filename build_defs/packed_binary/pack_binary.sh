#!/bin/bash -eu

# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

OUT="$(realpath "$1")"
UNCOMPRESSED="${OUT}.tmp.tar"
RUNFILES_PATH="$2"

cd "${RUNFILES_DIR:-}"

tar \
  --dereference \
  --exclude=MANIFEST \
  --sort=name \
  --owner=root:0 \
  --group=root:0 \
  --mtime='UTC 2019-01-01' \
  --create \
  . \
  --transform="s/./runfiles/" \
  -f \
  "${UNCOMPRESSED}"

echo "${RUNFILES_PATH}" > main
tar --append -f "${UNCOMPRESSED}" main

gzip -n < "${UNCOMPRESSED}" > "${OUT}"
