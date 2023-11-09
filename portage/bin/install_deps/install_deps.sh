#!/bin/bash -ex
# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

export ROOT="/${BOARD:+build/${BOARD}/}"
export SYSROOT="${ROOT}"
export PORTAGE_CONFIGROOT="${ROOT}"

install_deps() {
  local -i idx=0

  while [[ -v "INSTALL_ATOMS_TARGET_${idx}" ]]; do
    local -a atoms
    local current_group_var="INSTALL_ATOMS_TARGET_${idx}"

    read -ra atoms <<<"${!current_group_var}"
    if [[ "${#atoms[@]}" -gt 0 ]]; then
      # Use fakeroot on installing build dependencies since some files might
      # have non-root ownership or special permissions. Hopefully this does not
      # affect the result of building the package.
      # TODO: emerge is too slow! Find a way to speed up.
      #
      # We need to set ACCEPT_KEYWORDS to tell portage that 9999 packages are
      # allowed to be installed.
      time ACCEPT_KEYWORDS="~*" fakeroot emerge --oneshot --usepkgonly \
        --nodeps --noreplace --jobs "${atoms[@]}"
    fi
    unset "${current_group_var}"
    idx+=1
  done
}

install_deps
