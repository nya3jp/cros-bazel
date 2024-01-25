#!/bin/bash -ex
# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

export ROOT="/${BOARD:+build/${BOARD}/}"
export SYSROOT="${ROOT}"
export PORTAGE_CONFIGROOT="${ROOT}"

if [[ -d /home/root/chrome_root/src/third_party/depot_tools ]]; then
  # The src tarball has already had the hooks ran, so no need to run it in the
  # ebuild. It also won't run in the ebuild since the hooks need to access
  # the network.
  export USE="-runhooks ${USE}"
  # Use the CIPD cache provided by the tarball to avoid network access.
  export CIPD_CACHE_DIR="/home/root/chrome_root/.cipd-cache"

  # Tell the chrome ebuilds to use the local source.
  export CHROME_ORIGIN="LOCAL_SOURCE"
fi

if [[ -n "${USE_GOMA}" && "${USE_GOMA}" == "true" ]]; then
  tar --no-same-owner --no-same-permissions -xf /mnt/host/goma.tgz -C /tmp
  export GOMA_DIR=/tmp/goma-chromeos

  "${GOMA_DIR}/goma_ctl.py" start
fi

# cros_sdk creates this directory.
# This is needed for stage1 build because otherwise it fails trying to create
# /var/cache/distfiles/ccache/tmp
# TODO(b/277646771): Disable ccache to see if this can be removed.
mkdir -p /var/cache/chromeos-cache/distfiles

export FEATURES="${FEATURES} fakeroot"

exec "$@"
