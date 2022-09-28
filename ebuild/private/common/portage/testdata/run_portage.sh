#!/bin/bash
# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

set -e

archive_path="$1"
shift
if [[ -z "${archive_path}" ]]; then
  echo "usage: $0 <archive-path> <command...>"
  echo "example: $0 package-use.txt equery u pkg/x"
  exit 1
fi

base_dir="$(realpath -- "$(dirname -- "$0")")"
stage_dir="${base_dir}/stage"
stage_dir_in_chroot="/mnt/host/source/src/${stage_dir##*/src/}"

remove_stage_dir() {
  rm -rf "${stage_dir}"
}
remove_stage_dir
trap remove_stage_dir EXIT
"${base_dir}/unpack_spec.py" "${archive_path}" "${stage_dir}"

# Generate etc/make.conf to include the overlay in $PORTDIR_OVERLAY.
# TODO: Figure out why sourcing /etc/make.conf is necessary.
cat > "${stage_dir}/etc/make.conf" <<EOF
source /etc/make.conf
PORTDIR_OVERLAY="\${PORTDIR_OVERLAY} ${stage_dir_in_chroot}/overlay"
EOF

# Treat etc/make.conf/custom specially.
if [[ -f "${stage_dir}/etc/make.conf.custom" ]]; then
  echo "source ${stage_dir_in_chroot}/etc/make.conf.custom" >> "${stage_dir}/etc/make.conf"
fi

cros_sdk env FEATURES="digest" ROOT="${stage_dir_in_chroot}" SYSROOT="${stage_dir_in_chroot}" PORTAGE_CONFIGROOT="${stage_dir_in_chroot}" "$@"
