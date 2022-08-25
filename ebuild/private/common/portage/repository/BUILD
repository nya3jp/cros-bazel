load("@io_bazel_rules_go//go:def.bzl", "go_library")

go_library(
    name = "repository",
    srcs = [
        "repo.go",
        "reposet.go",
    ],
    importpath = "cros.local/bazel/ebuild/private/common/portage/repository",
    visibility = ["//visibility:public"],
    deps = [
        "//bazel/ebuild/private/common/standard/dependency",
        "//bazel/ebuild/private/common/standard/ebuild",
        "//bazel/ebuild/private/common/standard/packages",
        "//bazel/ebuild/private/common/standard/profile",
        "//bazel/ebuild/private/common/standard/version",
    ],
)
