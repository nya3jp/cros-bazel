load("@io_bazel_rules_go//go:def.bzl", "go_binary", "go_library")

go_binary(
    name = "build_ebuild",
    embed = [":build_ebuild_lib"],
    visibility = ["//visibility:public"],
)

go_library(
    name = "build_ebuild_lib",
    srcs = ["build_ebuild.go"],
    data = [
        ":run_in_container",
        "@dumb_init//file",
    ],
    importpath = "cros.local/ebuild/private",
    visibility = ["//visibility:private"],
    deps = [
        "//ebuild/private/portage/version",
        "@com_github_urfave_cli//:cli",
    ],
)

go_binary(
    name = "run_in_container",
    embed = [":run_in_container_lib"],
    visibility = ["//visibility:public"],
)

go_library(
    name = "run_in_container_lib",
    srcs = ["run_in_container.go"],
    importpath = "cros.local/ebuild/private",
    visibility = ["//visibility:private"],
    deps = [
        "@com_github_urfave_cli//:cli",
        "@org_golang_x_sys//unix",
    ],
)
