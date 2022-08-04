load("@bazel_gazelle//:def.bzl", "gazelle")
load("//ebuild:defs.bzl", "overlay_set")

# gazelle:prefix cros.local
gazelle(name = "gazelle")

overlay_set(
    name = "overlays",
    overlays = [
        "//third_party/overlay-amd64-generic",
        "//third_party/eclass-overlay",
        "//third_party/chromiumos-overlay",
        "//third_party/portage-stable",
    ],
    visibility = ["//visibility:public"],
)
