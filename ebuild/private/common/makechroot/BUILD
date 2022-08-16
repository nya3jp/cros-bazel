load("@io_bazel_rules_go//go:def.bzl", "go_library")

go_library(
    name = "makechroot",
    srcs = [
        "binpkgs.go",
        "overlay.go",
    ],
    importpath = "cros.local/ebuild/private/common/makechroot",
    visibility = ["//visibility:public"],
    deps = [
        "//ebuild/private/common/fileutil",
        "//ebuild/private/common/portage/xpak",
    ],
)
