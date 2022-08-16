load("@io_bazel_rules_go//go:def.bzl", "go_binary", "go_library")

go_library(
    name = "build_package_lib",
    srcs = ["main.go"],
    embedsrcs = ["setup.sh"],
    importpath = "cros.local/ebuild/private/cmd/build_package",
    visibility = ["//visibility:private"],
    deps = [
        "//ebuild/private/common/fileutil",
        "//ebuild/private/common/portage/version",
        "//ebuild/private/common/portage/xpak",
        "@com_github_alessio_shellescape//:shellescape",
        "@com_github_urfave_cli//:cli",
    ],
)

go_binary(
    name = "build_package",
    embed = [":build_package_lib"],
    visibility = ["//visibility:public"],
)
