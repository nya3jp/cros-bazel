# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/ebuild:defs.bzl", "sdk")

sdk(
    name = "sdk",
    src = "@chromiumos_sdk//file",
    board = "arm64-generic",
    host_deps = [
        "//bazel/prebuilts/amd64-host:fakeroot",
        "//bazel/prebuilts/amd64-host/cross-aarch64-cros-linux-gnu:binutils",
        "//bazel/prebuilts/amd64-host/cross-aarch64-cros-linux-gnu:compiler-rt",
        "//bazel/prebuilts/amd64-host/cross-aarch64-cros-linux-gnu:gcc",
        "//bazel/prebuilts/amd64-host/cross-aarch64-cros-linux-gnu:gdb",
        "//bazel/prebuilts/amd64-host/cross-aarch64-cros-linux-gnu:glibc",
        "//bazel/prebuilts/amd64-host/cross-aarch64-cros-linux-gnu:go",
        "//bazel/prebuilts/amd64-host/cross-aarch64-cros-linux-gnu:libcxx",
        "//bazel/prebuilts/amd64-host/cross-aarch64-cros-linux-gnu:libxcrypt",
        "//bazel/prebuilts/amd64-host/cross-aarch64-cros-linux-gnu:linux-headers",
        "//bazel/prebuilts/amd64-host/cross-aarch64-cros-linux-gnu:llvm-libunwind",
    ],
    target_deps = [
        "//bazel/prebuilts/arm64-generic/sys-kernel:linux-headers",
        "//bazel/prebuilts/arm64-generic/sys-libs:gcc-libs",
        "//bazel/prebuilts/arm64-generic/sys-libs:libcxx",
        "//bazel/prebuilts/arm64-generic/sys-libs:llvm-libunwind",
    ],
    visibility = ["//visibility:public"],
)
