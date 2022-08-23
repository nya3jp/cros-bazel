load("@io_bazel_rules_go//go:def.bzl", "go_library")

go_library(
    name = "packages",
    srcs = [
        "package.go",
        "stability.go",
    ],
    importpath = "cros.local/rules_ebuild/ebuild/private/common/standard/packages",
    visibility = ["//visibility:public"],
    deps = [
        "//ebuild/private/common/standard/dependency",
        "//ebuild/private/common/standard/makevars",
        "//ebuild/private/common/standard/version",
    ],
)
