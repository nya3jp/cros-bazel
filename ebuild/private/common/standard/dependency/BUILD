load("@io_bazel_rules_go//go:def.bzl", "go_library", "go_test")

go_library(
    name = "dependency",
    srcs = [
        "algorithm.go",
        "atom.go",
        "expr.go",
        "expr_compile.go",
    ],
    importpath = "cros.local/ebuild/private/common/standard/dependency",
    visibility = ["//visibility:public"],
    deps = [
        "//ebuild/private/common/standard/dependency/internal/grammer",
        "//ebuild/private/common/standard/naming",
        "//ebuild/private/common/standard/version",
    ],
)

go_test(
    name = "dependency_test",
    srcs = ["atom_test.go"],
    deps = [
        ":dependency",
        "//ebuild/private/common/standard/version",
    ],
)
