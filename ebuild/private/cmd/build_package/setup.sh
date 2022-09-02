#!/bin/bash -ex
# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# HACK: Print all outputs to stderr to avoid shuffled logs in Bazel output.
if [[ $# -gt 0 ]]; then
  exec >&2
fi

export ROOT="/build/${BOARD}/"
export SYSROOT="${ROOT}"
export PORTAGE_CONFIGROOT="${ROOT}"
export PORTAGE_USERNAME=root
export PORTAGE_GRPNAME=root
export RESTRICT="fetch"
export FEATURES="digest -sandbox -usersandbox"  # TODO: turn on sandbox

read -ra atoms <<<"${INSTALL_ATOMS_TARGET}"
if (( ${#atoms[@]} )); then
  # TODO: emerge is too slow! Find a way to speed up.
  time emerge --oneshot --usepkgonly --nodeps "${atoms[@]}"
fi

unset BOARD
unset INSTALL_ATOMS_TARGET

if [[ $# = 0 ]]; then
  exec bash
fi
exec "$@"
