#!/bin/bash -ex
# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# HACK: Print all outputs to stderr to avoid shuffled logs in Bazel output.
exec >&2

export PORTAGE_USERNAME=root
export PORTAGE_GRPNAME=root
export FEATURES="digest -sandbox -usersandbox"  # TODO: turn on sandbox

# TODO: Avoid hard-coding the default profile path.
ln -sf "/mnt/host/source/src/third_party/chromiumos-overlay/chromeos/config/make.conf.amd64-host" "/etc/make.conf"
ln -sf "/mnt/host/source/src/third_party/chromiumos-overlay/profiles/default/linux/amd64/10.0/sdk" "/etc/portage/make.profile"

# TODO: Generate make.conf.* automatically.
cat <<EOF > "/etc/make.conf.board_setup"
# Created by cros_sysroot_utils from --board=amd64-host.
ARCH="amd64"
BOARD_OVERLAY="/mnt/host/source/src/overlays/overlay-amd64-host"
BOARD_USE="amd64-host"
CHOST="x86_64-pc-linux-gnu"
MAKEOPTS="-j32"
PORTDIR_OVERLAY="/mnt/host/source/src/overlays/overlay-amd64-host"
EOF

cat <<EOF > "/etc/make.conf.host_setup"
# Automatically generated.  EDIT THIS AND BE SORRY.

FETCHCOMMAND="curl --ipv4 -f -y 30 --retry 9 -L --output \\"\\\${DISTDIR}/\\\${FILE}\\" \\"\\\${URI}\\""
RESUMECOMMAND="curl -C - --ipv4 -f -y 30 --retry 9 -L --output \\"\\\${DISTDIR}/\\\${FILE}\\" \\"\\\${URI}\\""

source /mnt/host/source/src/third_party/chromiumos-overlay/chromeos/config/make.conf.sdk-chromeos
FETCHCOMMAND_GS="/mnt/host/source/chromite/bin/gs_fetch_binpkg --boto \"/mnt/host/source/src/private-overlays/chromeos-overlay/googlestorage_account.boto\\" \\"\\\${URI}\\" \\"\\\${DISTDIR}/\\\${FILE}\\""
RESUMECOMMAND_GS="\${FETCHCOMMAND_GS}"
MAKEOPTS="-j128"
EOF

rm -f "/etc/make.conf.user"
cat <<EOF > "/etc/make.conf.user"
# no user config
EOF

mkdir -p "/build/${BOARD}/etc/portage"
# TODO: Avoid hard-coding the default profile path.
ln -sf "/mnt/host/source/src/third_party/chromiumos-overlay/chromeos/config/make.conf.generic-target" "/build/${BOARD}/etc/make.conf"
ln -sf "/etc/make.conf.user" "/build/${BOARD}/etc/make.conf.user"
ln -sf "/mnt/host/source/src/overlays/overlay-arm64-generic/profiles/base" "/build/${BOARD}/etc/portage/make.profile"

cat <<EOF > "/build/${BOARD}/etc/make.conf.board_setup"
# Created by cros_sysroot_utils from --board=arm64-generic.
ARCH="arm64"
BOARD_OVERLAY="/mnt/host/source/src/overlays/overlay-arm64-generic"
BOARD_USE="arm64-generic"
CHOST="aarch64-cros-linux-gnu"
MAKEOPTS="-j128"
PKG_CONFIG="/build/arm64-generic/build/bin/pkg-config"
PORTDIR_OVERLAY="/mnt/host/source/src/third_party/eclass-overlay
/mnt/host/source/src/third_party/portage-stable
/mnt/host/source/src/third_party/chromiumos-overlay
/mnt/host/source/src/overlays/overlay-arm64-generic"
ROOT="/build/arm64-generic/"
EOF

cat <<EOF > "/build/${BOARD}/etc/make.conf.board"
# AUTO-GENERATED FILE. DO NOT EDIT.

  # Source make.conf from each overlay.
BOTO_CONFIG="/mnt/host/source/src/private-overlays/chromeos-overlay/googlestorage_account.boto"
FETCHCOMMAND_GS="bash -c 'BOTO_CONFIG=/mnt/host/source/src/private-overlays/chromeos-overlay/googlestorage_account.boto /mnt/host/source/chromite/bin/gs_fetch_binpkg \\"\${URI}\\" \\"\${DISTDIR}/\${FILE}\\"'"
RESUMECOMMAND_GS="\$FETCHCOMMAND_GS"

# FULL_BINHOST is populated by the full builders. It is listed first because it
# is the lowest priority binhost. It is better to download packages from the
# postsubmit binhost because they are fresher packages.
PORTAGE_BINHOST="\$FULL_BINHOST"


# POSTSUBMIT_BINHOST is populated by the postsubmit builders. If the same
# package is provided by both the postsubmit and full binhosts, the package is
# downloaded from the postsubmit binhost.
source /mnt/host/source/src/third_party/chromiumos-overlay/chromeos/binhost/target/arm64-generic-POSTSUBMIT_BINHOST.conf
PORTAGE_BINHOST="\$PORTAGE_BINHOST \$POSTSUBMIT_BINHOST"
EOF

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
