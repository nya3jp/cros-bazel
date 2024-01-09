EAPI=7
KEYWORDS="*"
SLOT=0

IUSE="cros-host"
# Use of the exactly-one-of dependency is unnecessary here, but we want to
# ensure it's correctly parsed.
REQUIRED_USE="^^ ( cros-host )"
