load("//bazel/ebuild:defs.bzl", "overlay_set")

overlay_set(
    name = "overlays",
    overlays = [
        "//overlays/overlay-arm64-generic",
        "//third_party/eclass-overlay",
        "//third_party/chromiumos-overlay",
        "//third_party/portage-stable",
    ],
    visibility = ["//visibility:public"],
)
