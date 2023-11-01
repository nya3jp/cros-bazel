#!/bin/bash -x
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

set -eu -o pipefail

export ROOT="/${BOARD:+build/${BOARD}/}"
export SYSROOT="${ROOT}"
export PORTAGE_CONFIGROOT="${ROOT}"

# TODO(b/278728702): Figure out how this symlink gets created.
# glibc uses the SYMLINK_LIB environment variable to determine if it should
# create the /lib -> lib64 symlink. It doesn't handle the /usr/lib symlink
# though. There must be another package that is creating this, but I have
# yet to find it. I have searched baselayout as well, but it doesn't handle it.
if [[ "$(portageq envvar SYMLINK_LIB)" == "yes" ]]; then
  mkdir "${ROOT}/usr/lib64"
  ln -s lib64 "${ROOT}/usr/lib"
fi

# Create symlinks to do the same thing as src/scripts/build_sdk_board.
mkdir -p "${ROOT}/mnt/host"
ln -s /mnt/host/source/src/chromium/depot_tools "${ROOT}/mnt/host/depot_tools"

# Needed to tell chromite's cros_build_lib that we are running inside the
# SDK. We don't use a real version number since there is no such thing in the
# bazel world.
echo bazel > "${ROOT}/etc/cros_chroot_version"

# TODO: Find a way to share bash utils
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

# We duplicate the cleanup.rs functionality here because we need to
# run in the context of the container so we can have access to all the layer.
# If we try and create a tarball from the build_sdk command, we have lost the
# ephemeral base layers so we don't have a complete view of the filesystem.

PKG="${ROOT}var/db/pkg"
# CONTENTS: This file is sorted in the binpkg, but when portage installs the
#           binpkg it recreates it in a non-hermetic way, so we manually sort
#           it.
find "${PKG}" -name CONTENTS -exec sort -o '{}' '{}' \;
# environment.bz2: The environment contains EPOCHTIME and SRANDOM from when the
#                  package was installed. We could modify portage to omit these,
#                  but I didn't think the binpkg-hermetic FEATURE should apply
#                  to locally installed artifacts. So we just delete the file
#                  for now.
find "${PKG}" -name environment.bz2 -exec rm '{}' +

# COUNTER: Since we are installing packages in parallel the COUNTER variable
#          can change depending on when it was installed.
find "${PKG}" -name COUNTER -exec sed -i -e 'c 0' '{}' +

# We don't want tar to change the permissions on the root directory when
# we extract it.
chmod 755 "${ROOT}"

# We need to run fakeroot so the tarball contains the correct UIDs.
# We can't use --remove-files because we get overlayfs IO errors on some files.
# Exclude /usr/share/{doc,man} because they pull in a bunch of files we don't
# need. Evaluate setting INSTALL_MASK instead.
time fakeroot tar \
  --format gnu \
  --sort name \
  --mtime "1970-1-1 00:00Z" \
  --numeric-owner \
  --create \
  --directory "${ROOT}" \
  --exclude "./tmp/*" \
  --exclude "./var/cache/*" \
  --exclude "./packages" \
  --exclude "./build" \
  --exclude "./usr/share/doc/*" \
  --exclude "./usr/share/man/*" \
  --exclude="./etc/make.conf" \
  --exclude="./etc/make.conf.*" \
  --exclude="./etc/portage" \
  . | \
  zstd -3 --long -T0 --force -o "/mnt/host/.build_sdk/output.tar.zst"
