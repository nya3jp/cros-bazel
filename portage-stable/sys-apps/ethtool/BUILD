genrule(
  name = "ethtool",
  srcs = ["ethtool-4.13.ebuild", "@ethtool_4_13//file"],
  outs = ["ethtool-4.13.tbz2"],
  cmd = """
export TMPDIR="$$(mktemp -d /tmp/build_ebuild.XXXXXXXX)"
trap "rm -rf $${TMPDIR}" EXIT

export ROOT="$${TMPDIR}/root"
export SYSROOT="$${ROOT}"
export FEATURES="digest"
export PORTAGE_USERNAME="$$(id -un)"
export PORTAGE_GRPNAME="$$(id -gn)"
export PORTAGE_TMPDIR="$${TMPDIR}"
export DISTDIR="$${TMPDIR}/distfiles"
export PKGDIR="$${TMPDIR}/out"

set -x

mkdir -p "$${DISTDIR}"
cp $(location @ethtool_4_13//file) "$${DISTDIR}/ethtool-4.13.tar.xz"
$(location @chromiumos_portage_tool//:ebuild) $(location :ethtool-4.13.ebuild) clean package
cp "$${PKGDIR}/sys-apps/ethtool-4.13.tbz2" $@
""",
  tools = ["@chromiumos_portage_tool//:ebuild"],
)
