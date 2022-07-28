genrule(
  name = "hello",
  srcs = ["hello-1.0.ebuild"],
  outs = ["hello-1.0.tbz2"],
  cmd = """
export TMPDIR="$$(mktemp -d /tmp/build_ebuild.XXXXXXXX)"
trap "rm -rf $${TMPDIR}" EXIT

export FEATURES="-strict"
export PORTAGE_USERNAME="$$(id -un)"
export PORTAGE_GRPNAME="$$(id -gn)"
export PORTAGE_TMPDIR="$${TMPDIR}"
export PKGDIR="$${TMPDIR}/package_out"

$(location @chromiumos_portage_tool//:ebuild) $(location :hello-1.0.ebuild) clean package
cp "$${PKGDIR}/experimental/hello-1.0.tbz2" $@
""",
  tools = ["@chromiumos_portage_tool//:ebuild"],
)
