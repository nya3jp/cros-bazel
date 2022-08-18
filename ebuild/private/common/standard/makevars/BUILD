load("@io_bazel_rules_go//go:def.bzl", "go_library")

go_library(
    name = "makevars",
    srcs = [
        "parse.go",
        "vars.go",
    ],
    importpath = "cros.local/ebuild/private/common/standard/makevars",
    visibility = ["//visibility:public"],
    deps = [
        "@cc_mvdan_sh_v3//expand:go_default_library",
        "@cc_mvdan_sh_v3//syntax:go_default_library",
        "@com_github_alessio_shellescape//:shellescape",
    ],
)
