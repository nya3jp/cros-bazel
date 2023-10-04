EAPI=7
KEYWORDS="*"
SLOT=0

# taken from glibc ebuild
LOCALE_GEN_VER=2.10
SRC_URI+=" https://gitweb.gentoo.org/proj/locale-gen.git/snapshot/locale-gen-${LOCALE_GEN_VER}.tar.gz"
