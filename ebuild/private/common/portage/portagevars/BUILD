load("@io_bazel_rules_go//go:def.bzl", "go_library")

go_library(
    name = "portagevars",
    srcs = ["overlays.go"],
    importpath = "cros.local/bazel/ebuild/private/common/portage/portagevars",
    visibility = ["//visibility:public"],
    deps = ["//bazel/ebuild/private/common/standard/makevars"],
)
