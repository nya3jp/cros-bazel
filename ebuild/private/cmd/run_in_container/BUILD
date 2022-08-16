load("@io_bazel_rules_go//go:def.bzl", "go_binary", "go_library")

go_library(
    name = "run_in_container_lib",
    srcs = ["main.go"],
    importpath = "cros.local/ebuild/private/cmd/run_in_container",
    visibility = ["//visibility:private"],
    deps = [
        "@com_github_urfave_cli//:cli",
        "@org_golang_x_sys//unix",
    ],
)

go_binary(
    name = "run_in_container",
    embed = [":run_in_container_lib"],
    visibility = ["//visibility:public"],
)
