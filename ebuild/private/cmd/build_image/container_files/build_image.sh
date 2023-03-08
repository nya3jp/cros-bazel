#!/bin/bash
# Copyright 2023 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# Apply the base package override.
readonly base_image_util_path="/mnt/host/source/src/scripts/build_library/base_image_util.sh"
if [[ -n "${BASE_PACKAGE}" ]]; then
  sed -i 's,\${BASE_PACKAGE},'"${BASE_PACKAGE}"',g' "${base_image_util_path}"
fi
unset BASE_PACKAGE

# HACK: Rewrite base_image_util.sh to skip some steps we don't support yet.
# TODO: Remove these hacks.
sed -i 's,sudo "\${GCLIENT_ROOT}/chromite/licensing/licenses",true &,' "${base_image_util_path}"
sed -i 's,build_dlc,true &,' "${base_image_util_path}"
sed -i 's,create_dev_install_lists ,true &,' "${base_image_util_path}"
sed -i 's,"\${GCLIENT_ROOT}/chromite/scripts/pkg_size",true &,' "${base_image_util_path}"

/mnt/host/source/chromite/bin/build_image "$@"
RC=$?

set -ex

# TODO: scripts/build_image.sh builds a base image, then copies it and modifies
# the copy to build dev / test images. We should convert base / dev / test
# images to three seperate bazel rules, so that changing a dev image package
# doesn't rebuild the whole base image.
if [ "${RC}" -eq 0 ]; then
  chown "${HOST_UID}:${HOST_GID}" "/mnt/host/source/src/build/images/${BOARD}/latest/chromiumos_base_image.bin"
fi

# TODO: remove temporary files owned by root, handle sigint / sigterm so that we still remove those files on error.
exit "${RC}"
