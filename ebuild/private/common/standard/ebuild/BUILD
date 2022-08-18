load("@io_bazel_rules_go//go:def.bzl", "go_library")

go_library(
    name = "ebuild",
    srcs = [
        "ebuild_prelude.go",
        "processor.go",
    ],
    embedsrcs = ["ebuild_prelude.sh"],
    importpath = "cros.local/ebuild/private/common/standard/ebuild",
    visibility = ["//visibility:public"],
    deps = [
        "//ebuild/private/common/standard/makevars",
        "//ebuild/private/common/standard/version",
    ],
)
