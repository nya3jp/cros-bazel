#!/bin/bash -eu

# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

SRC="$1"
OUT="$2"

tmp=$(mktemp -d)

# deletes the temp directory
function cleanup {
  rm -rf "${tmp}"
}

# register the cleanup function to be called on the EXIT signal
trap cleanup EXIT

tar -xf "${SRC}" -C "${tmp}"
cd "$(dirname "${OUT}")"
OUT="$(basename "${OUT}")"
main=$(cat "${tmp}/main")
mv "${tmp}/runfiles" "${OUT}.runfiles"
cp "${OUT}.runfiles/${main}" "${OUT}"
