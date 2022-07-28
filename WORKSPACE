workspace(name = "rules_ebuild")

load("@bazel_tools//tools/build_defs/repo:git.bzl", "new_git_repository")
new_git_repository(
  name = "chromiumos_portage_tool",
  branch = "chromeos-2.3.75",
  remote = "https://chromium.googlesource.com/chromiumos/third_party/portage_tool",
  build_file = "@//:BUILD.chromiumos_portage_tool",
)
