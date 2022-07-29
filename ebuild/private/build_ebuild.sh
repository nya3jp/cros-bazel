#!/bin/bash

readonly runfiles_dir="$0.runfiles"

export TMPDIR="$(mktemp -d /tmp/build_ebuild.XXXXXXXX)"
trap "rm -rf ${TMPDIR}" EXIT

export ROOT="${TMPDIR}/root"
export SYSROOT="${ROOT}"
export FEATURES="digest"
export PORTAGE_USERNAME="$(id -un)"
export PORTAGE_GRPNAME="$(id -gn)"
export PORTAGE_TMPDIR="${TMPDIR}"
export DISTDIR="${TMPDIR}/distfiles"
export PKGDIR="${TMPDIR}/out"

set -x

mkdir -p "${DISTDIR}"
# cp $(location @ethtool_4_13//file) "${DISTDIR}/ethtool-4.13.tar.xz"

"${runfiles_dir}/chromiumos_portage_tool/ebuild "$1" clean package

cp "${PKGDIR}/sys-apps/ethtool-4.13.tbz2" $@
