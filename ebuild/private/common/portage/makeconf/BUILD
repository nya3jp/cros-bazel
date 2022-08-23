load("@io_bazel_rules_go//go:def.bzl", "go_library", "go_test")

go_library(
    name = "makeconf",
    srcs = ["makeconf.go"],
    importpath = "cros.local/rules_ebuild/ebuild/private/common/portage/makeconf",
    visibility = ["//visibility:public"],
    deps = [
        "@cc_mvdan_sh_v3//expand",
        "@cc_mvdan_sh_v3//syntax",
    ],
)

go_test(
    name = "makeconf_test",
    srcs = ["makeconf_test.go"],
    deps = [
        ":makeconf",
        "@com_github_google_go_cmp//cmp",
    ],
)
