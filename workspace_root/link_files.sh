#!/bin/bash -e
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# Usage: link_files.sh [alchemy|metallurgy]

link_files() {
  # ls can't find hidden files without picking up ..
  for f in $(find "$1" -maxdepth 1 -mindepth 1); do
    ln -s -v "${f}" .
  done
}

root=$(realpath "${BASH_SOURCE[0]}")
for i in $(seq 1 3); do
  root="$(dirname "${root}")"
done
cd "${root}"

for f in $(find . -maxdepth 1 -type l); do
  echo "Removing existing symlink ${f}"
  rm "${f}"
done

link_files "bazel/workspace_root/general"
link_files "bazel/workspace_root/${1:-alchemy}"
ln -s -v bazel/rules_cros .
