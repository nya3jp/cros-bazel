load("@io_bazel_rules_go//go:def.bzl", "go_binary", "go_library")

go_binary(
    name = "build_ebuild",
    data = [
        "@chromiumos_portage_tool//:ebuild",
    ],
    embed = [":build_ebuild_lib"],
    visibility = ["//visibility:public"],
)

go_library(
    name = "build_ebuild_lib",
    srcs = ["build_ebuild.go"],
    importpath = "cros.local/ebuild/private",
    visibility = ["//visibility:private"],
    deps = [
        "//ebuild/private/go/portage/version",
        "@com_github_urfave_cli//:cli",
    ],
)
