load("@io_bazel_rules_go//go:def.bzl", "go_binary", "go_library")

go_binary(
    name = "build_sdk",
    embed = [":build_sdk_lib"],
    visibility = ["//visibility:public"],
)

go_library(
    name = "build_sdk_lib",
    srcs = ["main.go"],
    embedsrcs = ["setup.sh"],
    importpath = "cros.local/ebuild/private/cmd/build_sdk",
    visibility = ["//visibility:private"],
    deps = [
        "//ebuild/private/common/fileutil",
        "//ebuild/private/common/makechroot",
        "@com_github_urfave_cli//:cli",
    ],
)
