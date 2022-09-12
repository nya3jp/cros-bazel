#!/bin/bash -ex
# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# HACK: Print all outputs to stderr to avoid shuffled logs in Bazel output.
exec >&2

export PORTAGE_USERNAME=root
export PORTAGE_GRPNAME=root
export RESTRICT="fetch"
export FEATURES="digest -sandbox -usersandbox"  # TODO: turn on sandbox

for i in /stage/tarballs/*; do
  tar -xv -f "${i}" -C /
done

# TODO: Consider using fakeroot-like approach to emulate file permissions.
sed -i -e '/dir_mode_map = {/,/}/s/False/True/' /usr/lib/python3.6/site-packages/portage/package/ebuild/config.py

# HACK: Do not use namespaces in ebuild(1).
# TODO: Find a better way.
sed -i "/keywords\['unshare/d" /usr/lib/python3.6/site-packages/portage/package/ebuild/doebuild.py

read -ra atoms <<<"${INSTALL_ATOMS_HOST}"
if (( ${#atoms[@]} )); then
  # TODO: emerge is too slow! Find a way to speed up.
  time emerge --oneshot --usepkgonly --nodeps --jobs=16 "${atoms[@]}"
fi

read -ra atoms <<<"${INSTALL_ATOMS_TARGET}"
if (( ${#atoms[@]} )); then
  # TODO: emerge is too slow! Find a way to speed up.
  time ROOT="/build/${BOARD}/" SYSROOT="/build/${BOARD}/" PORTAGE_CONFIGROOT="/build/${BOARD}/" emerge --oneshot --usepkgonly --nodeps --jobs=16 "${atoms[@]}"
fi

# TODO: This is a wrong way to set up cross compilers!!!
rsync -a /usr/*-cros-linux-gnu/ "/build/${BOARD}/"
