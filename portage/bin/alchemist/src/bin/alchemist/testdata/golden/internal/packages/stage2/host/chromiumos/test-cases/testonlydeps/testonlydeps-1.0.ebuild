EAPI=7
KEYWORDS="*"
SLOT=0

# `ebuild` should depende on `simple/aaa`
# and `ebuild_test` should depende on both `simple/aaa` and `simple/bbb`.

IUSE="test"

DEPEND="
    simple/aaa
    test? ( simple/bbb )
"
