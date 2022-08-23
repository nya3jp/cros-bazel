load("@io_bazel_rules_go//go:def.bzl", "go_binary", "go_library")

go_library(
    name = "build_package_lib",
    srcs = ["main.go"],
    embedsrcs = ["setup.sh"],
    importpath = "cros.local/rules_ebuild/ebuild/private/cmd/build_package",
    visibility = ["//visibility:private"],
    deps = [
        "//ebuild/private/common/fileutil",
        "//ebuild/private/common/portage/xpak",
        "//ebuild/private/common/runfiles",
        "//ebuild/private/common/standard/version",
        "@com_github_alessio_shellescape//:shellescape",
        "@com_github_urfave_cli//:cli",
        "@io_bazel_rules_go//go/tools/bazel:go_default_library",
    ],
)

go_binary(
    name = "build_package",
    data = [
        "//ebuild/private/cmd/run_in_container",
    ],
    embed = [":build_package_lib"],
    visibility = ["//visibility:public"],
)
