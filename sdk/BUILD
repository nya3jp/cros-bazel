load("//ebuild:defs.bzl", "sdk")

sdk(
    name = "sdk",
    src = "@chromiumos_sdk//file",
    board = "arm64-generic",
    host_deps = [
        "//third_party/prebuilts/host:fakeroot",
        "//third_party/prebuilts/host/cross-aarch64-cros-linux-gnu:binutils",
        "//third_party/prebuilts/host/cross-aarch64-cros-linux-gnu:compiler-rt",
        "//third_party/prebuilts/host/cross-aarch64-cros-linux-gnu:gcc",
        "//third_party/prebuilts/host/cross-aarch64-cros-linux-gnu:gdb",
        "//third_party/prebuilts/host/cross-aarch64-cros-linux-gnu:glibc",
        "//third_party/prebuilts/host/cross-aarch64-cros-linux-gnu:go",
        "//third_party/prebuilts/host/cross-aarch64-cros-linux-gnu:libcxx",
        "//third_party/prebuilts/host/cross-aarch64-cros-linux-gnu:libxcrypt",
        "//third_party/prebuilts/host/cross-aarch64-cros-linux-gnu:linux-headers",
        "//third_party/prebuilts/host/cross-aarch64-cros-linux-gnu:llvm-libunwind",
    ],
    target_deps = [
        "//third_party/prebuilts/arm64-generic/sys-kernel:linux-headers",
        "//third_party/prebuilts/arm64-generic/sys-libs:gcc-libs",
        "//third_party/prebuilts/arm64-generic/sys-libs:libcxx",
        "//third_party/prebuilts/arm64-generic/sys-libs:llvm-libunwind",
    ],
    visibility = ["//visibility:public"],
)
