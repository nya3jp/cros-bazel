# Copyright 2023 The ChromiumOS Authors
# Distributed under the terms of the GNU General Public License v2

EAPI="7"
LICENSE="metapackage"
SLOT="0"
KEYWORDS="*"

# Primordial packages
RDEPEND="
	virtual/os-headers
	sys-libs/glibc
	sys-libs/libcxx
	sys-libs/llvm-libunwind
"

# A host tool
RDEPEND+="
	sys-devel/binutils
"

# We need a go compiler to compile go.
RDEPEND+="
	dev-lang/go
"
