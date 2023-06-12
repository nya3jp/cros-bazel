# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# To regenerate extract-test-1.2.3.tbz2, copy or symlink this file to:
#   ~/chromiumos/src/third_party/chromiumos-overlay/virtual/extract-test/
# and run:
#   sudo emerge --nodeps virtual/extract-test
# Then the binary package will be saved at:
#   /var/lib/portage/pkgs/virtual/extract-test-1.2.3.tbz2

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
  for p in {,/usr}/{bin,lib}; do
    exeinto "${p}"
    doexe helloworld
  done
}
