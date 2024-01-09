EAPI=7
KEYWORDS="*"
SLOT=0

if [[ "${CATEGORY}" != cross-* ]]; then
	BDEPEND="sys-devel/binutils sys-devel/gcc"
fi

PDEPEND="test-cases/inherit"
