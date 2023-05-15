# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load(":toolchain.bzl", "generate_cc_toolchains")

package(default_visibility = ["//visibility:public"])

generate_cc_toolchains()

# Workaround for https://github.com/bazelbuild/bazel/issues/12712
cc_toolchain_suite(
    name = "cros-cc-toolchain-suite",
    toolchains = {
        "x86_64": ":x86_64-pc-linux-gnu",
        "k8": ":x86_64-pc-linux-gnu",
    },
)
