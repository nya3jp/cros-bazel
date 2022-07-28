workspace(name = "rules_ebuild")

load("@bazel_tools//tools/build_defs/repo:git.bzl", "new_git_repository")
new_git_repository(
  name = "chromiumos_portage_tool",
  branch = "chromeos-2.3.75",
  remote = "https://chromium.googlesource.com/chromiumos/third_party/portage_tool",
  build_file = "@//:BUILD.chromiumos_portage_tool",
)

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")

http_file(
    name = "ethtool_4_13",
    urls = ["https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/ethtool-4.13.tar.xz"],
    sha256 = "b7c1a380007d30eaf261a63b3cfc000f9d93f9eb7626dcd48b5d2a733af99cba",
)
