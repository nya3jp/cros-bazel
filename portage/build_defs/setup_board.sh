#!/bin/bash -ue

# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

if [[ ! -e /etc/cros_chroot_version ]]; then
  echo "Cannot setup board outside the cros SDK chroot."
  exit 1
fi

print_usage_and_exit() {
  exec >&2
  echo "usage: $0 [flags]"
  echo "   -b board - Name of the board"
  echo "   -o output_file - Output file location."
  exit 1
}

while getopts "b:o:" OPTNAME; do
  case "${OPTNAME}" in
    b) BOARD="${OPTARG}";;
    o) OUTPUT="${OPTARG}";;
    *) print_usage_and_exit;;
  esac
done

/mnt/host/source/chromite/bin/setup_board \
  -b "${BOARD}" \
  --reuse-configs \
  --force \
  --skip-chroot-upgrade

touch "${OUTPUT}"
