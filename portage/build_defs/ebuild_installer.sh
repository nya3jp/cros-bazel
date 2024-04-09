#!/bin/bash -ue

# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

if [[ ! -e /etc/cros_chroot_version ]]; then
  echo "Cannot build ':installed' targets outside the cros SDK chroot."
  exit 1
fi

print_usage_and_exit() {
  exec >&2
  echo "usage: $0 [flags] [dep checksums]"
  echo "   -b file - Binary package to install"
  echo "   -d dir - Directory to place the binary package"
  echo "   -e cmd - Emerge command to execute"
  echo "   -c file - File to write checksum into"
  echo "   -s file - File to checksum"
  echo "   -t file - path to the xpaktool"
  echo "   -x key=val - Xpak entries to update"
  exit 1
}

CHECKSUM_FILES=()
XPAK_UPDATES=()
while getopts "b:d:e:c:s:t:x:" OPTNAME; do
  case "${OPTNAME}" in
    b) BINPKG="${OPTARG}";;
    d) DEST="${OPTARG}";;
    e) EMERGE_CMD="${OPTARG}";;
    c) CHECKSUM="${OPTARG}";;
    s) CHECKSUM_FILES+=("${OPTARG}");;
    t) XPAK_TOOL="${OPTARG}";;
    x) XPAK_UPDATES+=("${OPTARG}");;
    *) print_usage_and_exit;;
  esac
done

sudo mkdir -p "$(dirname "${DEST}")"

# Ideally we'd just use a symlink here for performance reasons. However, if you
# create a symlink for package foo, then foo emerges just fine, but any package
# that rdepends on it will fail to detect it during emerge.
# We use a tmp file so portage doesn't pick up the partial binpkg.
sudo cp "${BINPKG}" "${DEST}.tmp"
sudo "${XPAK_TOOL}" update-xpak --binpkg "${DEST}.tmp" "${XPAK_UPDATES[@]}"
sudo chmod 644 "${DEST}.tmp"
sudo mv "${DEST}.tmp" "${DEST}"

# Calculate the checksums in parallel with emerging.
(cat "${BINPKG}" "${CHECKSUM_FILES[@]}" | sha256sum > "${CHECKSUM}" ) &

# Note: Despite only installing a single package at a time, this operates in
# parallel because emerge doesn't take a lock and allows you to install multiple
# packages at the same time.
# Since we divide this up into one action per package, bazel can automatically
# handle the installation order so that a package can only install once the deps
# are installed.
${EMERGE_CMD}

wait
