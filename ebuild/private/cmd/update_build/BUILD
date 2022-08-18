load("@io_bazel_rules_go//go:def.bzl", "go_binary", "go_library")

go_library(
    name = "update_build_lib",
    srcs = ["main.go"],
    importpath = "cros.local/ebuild/private/cmd/update_build",
    visibility = ["//visibility:private"],
    deps = [
        "//ebuild/private/common/standard/version",
        "@com_github_urfave_cli//:cli",
    ],
)

go_binary(
    name = "update_build",
    embed = [":update_build_lib"],
    visibility = ["//visibility:public"],
)
