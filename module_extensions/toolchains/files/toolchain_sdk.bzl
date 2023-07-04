# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/build_defs:filter_files.bzl", "filter_files")

visibility("//bazel/module_extensions/toolchains/files/...")

def toolchain_sdk_filter_files(**kwargs):
    filter_files(
        srcs = ["@toolchain_sdk//:sysroot_files"],
        strip_prefix = "../_main~toolchains~toolchain_sdk",
        **kwargs
    )
