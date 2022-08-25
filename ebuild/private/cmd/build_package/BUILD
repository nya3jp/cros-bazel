load("@io_bazel_rules_go//go:def.bzl", "go_binary", "go_library")

go_library(
    name = "build_package_lib",
    srcs = ["main.go"],
    embedsrcs = ["setup.sh"],
    importpath = "cros.local/bazel/ebuild/private/cmd/build_package",
    visibility = ["//visibility:private"],
    deps = [
        "//bazel/ebuild/private/common/fileutil",
        "//bazel/ebuild/private/common/portage/xpak",
        "//bazel/ebuild/private/common/runfiles",
        "//bazel/ebuild/private/common/standard/version",
        "@com_github_alessio_shellescape//:shellescape",
        "@com_github_urfave_cli//:cli",
        "@io_bazel_rules_go//go/tools/bazel:go_default_library",
    ],
)

go_binary(
    name = "build_package",
    data = [
        "//bazel/ebuild/private/cmd/run_in_container",
    ],
    embed = [":build_package_lib"],
    visibility = ["//visibility:public"],
)
