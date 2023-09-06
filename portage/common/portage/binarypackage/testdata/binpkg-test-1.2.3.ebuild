# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# To regenerate binpkg-test-1.2.3.tbz2, copy or symlink this file to:
#   ~/chromiumos/src/third_party/chromiumos-overlay/sys-apps/binpkg-test/
# and run:
#   sudo emerge --nodeps sys-apps/binpkg-test
# Then the binary package will be saved at:
#   /var/lib/portage/pkgs/sys-apps/binpkg-test-1.2.3.tbz2

EAPI=7
SLOT=0
KEYWORDS="*"
LICENSE="BSD-Google"

src_unpack() {
  mkdir -p "${S}"
}

src_compile() {
  cat <<EOF > hello
#!/bin/sh
echo "Hello, world!"
EOF
  chmod +x hello
}

src_install() {
  exeinto /usr/bin
  exeopts -o 123 -g 234
  doexe hello
}
