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
