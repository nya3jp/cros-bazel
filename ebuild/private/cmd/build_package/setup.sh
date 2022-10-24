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
export FEATURES="fakeroot -sandbox -usersandbox -ipc-sandbox -mount-sandbox -network-sandbox -pid-sandbox"
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

if [[ $# = 0 ]]; then
  exec bash
fi
exec "$@"
