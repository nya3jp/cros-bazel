load("@io_bazel_rules_go//go:def.bzl", "go_library", "go_test")

go_library(
    name = "grammer",
    srcs = ["grammer.go"],
    importpath = "cros.local/ebuild/private/common/standard/dependency/internal/grammer",
    visibility = ["//ebuild/private/common/standard/dependency:__subpackages__"],
    deps = [
        "@com_github_alecthomas_participle_v2//:participle",
        "@com_github_alecthomas_participle_v2//lexer",
    ],
)

go_test(
    name = "grammer_test",
    srcs = ["grammer_test.go"],
    deps = [
        ":grammer",
        "@com_github_google_go_cmp//cmp",
    ],
)
