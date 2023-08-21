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

# b/296450672 - Disable the compiler wrappers to see if this fixes the problem.
if [[ -f /mnt/host/source/src/third_party/chromiumos-overlay/profiles/base/profile.bashrc ]]; then
  sed -i -E -e '/cros_pre_src_prepare_build_toolchain_catch\(\)/a return' /mnt/host/source/src/third_party/chromiumos-overlay/profiles/base/profile.bashrc
fi

# b/296450672 - Ensure x86_64-cros-linux-gnu-* exist.
set +e
mount
which x86_64-cros-linux-gnu-gcc
/usr/bin/x86_64-cros-linux-gnu-gcc --version
ls -l /usr/bin/x86_64-cros-linux-gnu-*
ls -lH /usr/bin/x86_64-cros-linux-gnu-*

exec "$@"
