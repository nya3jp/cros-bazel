#!/bin/bash -ex
# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# HACK: Print all outputs to stderr to avoid shuffled logs in Bazel output.
if [[ $# -gt 0 ]]; then
  exec >&2
fi

export LANG=en_US.UTF-8
export ROOT="/build/${BOARD}/"
export SYSROOT="${ROOT}"
export PORTAGE_CONFIGROOT="${ROOT}"
export PORTAGE_USERNAME=root
export PORTAGE_GRPNAME=root
export RESTRICT="fetch"
export FEATURES="-sandbox -usersandbox -ipc-sandbox -mount-sandbox -network-sandbox -pid-sandbox"
export CCACHE_DISABLE=1

install_deps() {
  local -i idx=0

  while [[ -v "INSTALL_ATOMS_TARGET_${idx}" ]]; do
    local -a atoms
    local current_group_var="INSTALL_ATOMS_TARGET_${idx}"

    read -ra atoms <<<"${!current_group_var}"
    if [[ "${#atoms[@]}" -gt 0 ]]; then
      # We need to unmask the -9999 cros-workon ebuilds so we can install them
      mkdir -p "${ROOT}/etc/portage/package.accept_keywords"
      printf "%s\n" "${atoms[@]}" \
        >> "${ROOT}/etc/portage/package.accept_keywords/cros-workon"
      # Use fakeroot on installing build dependencies since some files might
      # have non-root ownership or special permissions. Hopefully this does not
      # affect the result of building the package.
      # TODO: emerge is too slow! Find a way to speed up.
      time fakeroot emerge --oneshot --usepkgonly --nodeps --noreplace --jobs \
        "${atoms[@]}"
    fi
    unset "${current_group_var}"
    idx+=1
  done
}

install_deps

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
fi

export FEATURES="${FEATURES} fakeroot"

if [[ $# = 0 ]]; then
  exec bash
fi
exec "$@"
