load("@io_bazel_rules_go//go:def.bzl", "go_binary", "go_library")

go_library(
    name = "run_in_container_lib",
    srcs = ["main.go"],
    importpath = "cros.local/bazel/ebuild/private/cmd/run_in_container",
    visibility = ["//visibility:private"],
    deps = [
        "//bazel/ebuild/private/common/runfiles",
        "@com_github_urfave_cli//:cli",
        "@io_bazel_rules_go//go/tools/bazel:go_default_library",
        "@org_golang_x_sys//unix",
    ],
)

go_binary(
    name = "run_in_container",
    data = [
        "//bazel/prebuilts:squashfuse",
        "@dumb_init//file",
    ],
    embed = [":run_in_container_lib"],
    visibility = ["//visibility:public"],
)
