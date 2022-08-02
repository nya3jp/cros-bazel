genrule(
    name = "squashfs",
    srcs = ["@chromiumos_sdk//file"],
    outs = ["chromiumos_sdk.squashfs"],
    cmd = "xzcat $< | mksquashfs - $@ -tar -all-time 0",
    visibility = ["//visibility:public"],
)
