load("@io_bazel_rules_go//go:def.bzl", "go_library", "go_test")

go_library(
    name = "version",
    srcs = ["version.go"],
    importpath = "cros.local/ebuild/private/go/portage/version",
    visibility = ["//ebuild:__subpackages__"],
)

go_test(
    name = "version_test",
    srcs = ["version_test.go"],
    deps = [":version"],
)
