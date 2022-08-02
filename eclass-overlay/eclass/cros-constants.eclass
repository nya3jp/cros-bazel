# Copyright (c) 2013 The Chromium OS Authors. All rights reserved.
# Distributed under the terms of the GNU General Public License v2

# @ECLASS: cros-constants.eclass
# @MAINTAINER:
# ChromiumOS Build Team
# @BUGREPORTS:
# Please report bugs via http://crbug.com/new (with label Build)
# @VCSURL: https://chromium.googlesource.com/chromiumos/overlays/chromiumos-overlay/+/master/eclass/@ECLASS@
# @BLURB: helper eclass with various useful constants for building ChromeOS
# @DESCRIPTION:
# A collection of various useful constants for building ChromeOS


# @ECLASS-VARIABLE: CROS_GIT_HOST_URL
# @DESCRIPTION:
# Url for the git server containing various Chromium/OS repos.
CROS_GIT_HOST_URL="https://chromium.googlesource.com"

# @ECLASS-VARIABLE: CROS_GIT_INT_HOST_URL
# @DESCRIPTION:
# Url for the git server containing various Chrome/OS repos.
CROS_GIT_INT_HOST_URL="https://chrome-internal.googlesource.com"

# @ECLASS-VARIABLE: CROS_GIT_AOSP_URL
# @DESCRIPTION:
# Url for the git server containing various Android repos.
CROS_GIT_AOSP_URL="https://android.googlesource.com"

# @ECLASS-VARIABLE: AUTOTEST_BASE
# @DESCRIPTION:
# Path to build-time destination of autotest (relative to sysroot).
AUTOTEST_BASE="/usr/local/build/autotest"

# @ECLASS-VARIABLE: CHROOT_SOURCE_ROOT
# @DESCRIPTION:
# Path to location of source code in the chroot.
# Ebuilds should not override this.  It is controllable so the overall build
# system can redirect to a different location.  For example, using portage
# prefix allows building without a chroot in which case this path as-is would
# never work.
: ${CHROOT_SOURCE_ROOT:='/mnt/host/source'}

# @ECLASS-VARIABLE: CHROMITE_DIR
# @DESCRIPTION:
# Path to location of chromite source code in the chroot.
CHROMITE_DIR="${CHROOT_SOURCE_ROOT}/chromite"

# @ECLASS-VARIABLE: CHROMITE_BIN_DIR
# @DESCRIPTION:
# Path to location of chromite executable directory in the chroot.
CHROMITE_BIN_DIR="${CHROMITE_DIR}/bin"

# @ECLASS-VARIABLE: ARC_PREFIX
# @DESCRIPTION:
# Path to root directory of ARC++ install (relative to sysroot).
ARC_PREFIX="/opt/google/containers/android"

# @ECLASS-VARIABLE: ARC_VENDOR_DIR
# @DESCRIPTION:
# Path to install directory for temporary /vendor build files (relative to
# sysroot).
ARC_VENDOR_DIR="/build/rootfs${ARC_PREFIX}/vendor"

# @ECLASS-VARIABLE: ARC_ETC_DIR
# @DESCRIPTION:
# Path to install directory for temporary /etc build files (relative to
# sysroot).
ARC_ETC_DIR="/build/rootfs${ARC_PREFIX}/etc"

