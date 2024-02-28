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

  # HACK: Run `git init` to make chrome/src a git repository. `dirmd read`
  # needs this to know where the chrome source root directory is.
  # TODO(b/327296393): Remove this hack.
  if type git; then
    git -C /home/root/chrome_root/src init
  fi
fi

# cros_sdk creates this directory.
# This is needed for stage1 build because otherwise it fails trying to create
# /var/cache/distfiles/ccache/tmp
# TODO(b/277646771): Disable ccache to see if this can be removed.
mkdir -p /var/cache/chromeos-cache/distfiles

export FEATURES="${FEATURES} fakeroot"

exec "$@"
