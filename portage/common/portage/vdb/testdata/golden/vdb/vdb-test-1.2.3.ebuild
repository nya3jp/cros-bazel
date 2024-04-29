# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# To regenerate vdb-test-1.2.3.tbz2, copy or symlink this file to:
#   ~/chromiumos/src/third_party/chromiumos-overlay/virtual/vdb-test/
# and run:
#   sudo emerge --nodeps virtual/vdb-test
# Then the binary package will be saved at:
#   /var/lib/portage/pkgs/virtual/vdb-test-1.2.3.tbz2

EAPI=7
SLOT=0
KEYWORDS="*"

# We don't actually consume any of this. We just need it so we get the
# cros_workon_ IUSE flag.
CROS_WORKON_COMMIT="c5ee2d43b1aae1af66b773411ffb2164618ec027"
CROS_WORKON_TREE="00bcf6dfb2521c35692d5a905add7f116b1d0595"
CROS_WORKON_PROJECT="chromiumos/platform2"
CROS_WORKON_LOCALNAME="platform2"
CROS_WORKON_DESTDIR="${S}/platform2"
CROS_WORKON_INCREMENTAL_BUILD=1
CROS_WORKON_SUBTREE="chromeos-config"

inherit cros-workon

src_unpack() {
  mkdir -p "${S}"
}

src_compile() {
  cat <<EOF > helloworld
#!/bin/sh
echo "Hello, world!"
EOF
  chmod +x helloworld
}

src_install() {
  exeinto /usr/bin
  doexe helloworld
  dosym helloworld /usr/bin/helloworld.symlink
}
