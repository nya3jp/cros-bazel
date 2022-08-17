load("@io_bazel_rules_go//go:def.bzl", "go_binary", "go_library")

go_library(
    name = "generate_build_lib",
    srcs = ["main.go"],
    importpath = "cros.local/ebuild/private/cmd/generate_build",
    visibility = ["//visibility:private"],
    deps = [
        "//ebuild/private/common/fileutil",
        "//ebuild/private/common/portage/version",
        "@com_github_urfave_cli//:cli",
    ],
)

go_binary(
    name = "generate_build",
    embed = [":generate_build_lib"],
    visibility = ["//visibility:public"],
)
