#!/bin/bash -ex
# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# TODO: Consider replacing this shell script with Go.

# cros_sdk will bind mount depot_tools to /mnt/host/depot_tools. This is only
# needed for chrome and chrome-icu. Since chromium includes depot_tools, we can
# just use that.
if [[ -d /home/root/chrome_root/src/third_party/depot_tools ]]; then
  mkdir -p /mnt/host
  ln -s /home/root/chrome_root/src/third_party/depot_tools /mnt/host/depot_tools
  # The src tarball has already had the hooks ran, so no need to run it in the
  # ebuild. It also won't run in the ebuild since the hooks need to access
  # the network.
  export USE="-runhooks ${USE}"
  # Use the CIPD cache provided by the tarball to avoid network access.
  export CIPD_CACHE_DIR="/home/root/chrome_root/.cipd-cache"

  # Tell the chrome ebuilds to use the local source.
  export CHROME_ORIGIN="LOCAL_SOURCE"
fi

export FEATURES="${FEATURES} fakeroot"

# Additional logging to debug b/294912568
# TODO(b/294912568): Remove this.
if [[ "$@[*]" == */mnt/host/source/src/third_party/chromiumos-overlay/sys-libs/llvm-libunwind/llvm-libunwind-*.ebuild* ]]; then
  ls -la /mnt/host/source/src
  ls -la /mnt/host/source/src/third_party
  ls -la /mnt/host/source/src/third_party/llvm-project
  ls -la /mnt/host/source/src/third_party/llvm-project/.git
fi

exec "$@"
