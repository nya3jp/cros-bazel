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

# `src_unpack` will run as `${PORTAGE_USERNAME}` if specified, or `portage`
# otherwise. If we don't set this variable, we get UID mismatches between the
# `.git` repos and the user running `src_unpack`.
#
# Normally `PORTAGE_USERNAME` is set when entering the chroot, but since we
# are executing as a bazel action, the environment has been stripped of all
# variables.
#
# See b/394378820#comment22.
export PORTAGE_USERNAME
PORTAGE_USERNAME="$(id --user --name)"

/mnt/host/source/chromite/bin/setup_board \
  -b "${BOARD}" \
  --reuse-configs \
  --force \
  --skip-chroot-upgrade

touch "${OUTPUT}"
