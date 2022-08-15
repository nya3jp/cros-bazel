load("@io_bazel_rules_go//go:def.bzl", "go_binary", "go_library")

go_binary(
    name = "build_ebuild",
    embed = [":build_ebuild_lib"],
    visibility = ["//visibility:public"],
)

go_library(
    name = "build_ebuild_lib",
    srcs = ["build_ebuild.go"],
    embedsrcs = ["build_ebuild_setup.sh"],
    importpath = "cros.local/ebuild/private",
    visibility = ["//visibility:private"],
    deps = [
        "//ebuild/private/portage/version",
        "//ebuild/private/portage/xpak",
        "@com_github_alessio_shellescape//:shellescape",
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

go_binary(
    name = "create_squashfs",
    embed = [":create_squashfs_lib"],
    visibility = ["//visibility:public"],
)

go_library(
    name = "create_squashfs_lib",
    srcs = ["create_squashfs.go"],
    importpath = "cros.local/ebuild/private",
    visibility = ["//visibility:private"],
    deps = [
        "@com_github_urfave_cli//:cli",
    ],
)
