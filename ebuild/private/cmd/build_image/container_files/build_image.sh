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

# HACK: Rewrite test_image_util.sh to skip some steps we don't support yet.
# TODO: Remove these hacks.
readonly test_image_util_path="/mnt/host/source/src/scripts/build_library/test_image_util.sh"
sed -i 's,build_dlc,true &,' "${test_image_util_path}"

exec /mnt/host/source/chromite/bin/build_image "$@"
