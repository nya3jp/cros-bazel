genrule(
    name = "convert_to_squashfs",
    srcs = ["@chromiumos_sdk//file"],
    outs = ["chromiumos_sdk.squashfs"],
    # TODO: Avoid -all-root.
    cmd = "xzcat $< | mksquashfs - $@ -tar -all-time 0 -all-root",
)

alias(
    name = "squashfs",
    actual = ":convert_to_squashfs",
    visibility = ["//visibility:public"],
)
