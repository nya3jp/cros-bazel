#!/bin/bash -ex
# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

export ROOT="/${BOARD:+build/${BOARD}/}"
export SYSROOT="${ROOT}"
export PORTAGE_CONFIGROOT="${ROOT}"

if [[ -d /home/root/chrome_root/src/third_party/depot_tools ]]; then
  # Use the CIPD cache provided by the tarball to avoid network access.
  export CIPD_CACHE_DIR="/home/root/chrome_root/.cipd-cache"

  # Tell the chrome ebuilds to use the local source.
  export CHROME_ORIGIN="LOCAL_SOURCE"
fi

export FEATURES="${FEATURES} fakeroot"

exec "$@"
