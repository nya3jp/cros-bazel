load("@io_bazel_rules_go//go:def.bzl", "go_library")

go_library(
    name = "repository",
    srcs = [
        "repo.go",
        "reposet.go",
    ],
    importpath = "cros.local/rules_ebuild/ebuild/private/common/portage/repository",
    visibility = ["//visibility:public"],
    deps = [
        "//ebuild/private/common/standard/dependency",
        "//ebuild/private/common/standard/ebuild",
        "//ebuild/private/common/standard/packages",
        "//ebuild/private/common/standard/profile",
        "//ebuild/private/common/standard/version",
    ],
)
