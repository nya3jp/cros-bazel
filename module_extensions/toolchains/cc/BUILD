# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_skylib//lib:selects.bzl", "selects")
load("@bazel_skylib//rules:common_settings.bzl", "bool_flag")
load("@rules_cc//cc:defs.bzl", "cc_toolchain_suite")
load(":toolchain.bzl", "generate_cc_toolchains")

package(default_visibility = ["//visibility:public"])

bool_flag(
    name = "hermetic",
    # TODO: Switch this to true once it's stable.
    build_setting_default = False,
)

config_setting(
    name = "hermetic_explicitly_enabled",
    flag_values = {
        ":hermetic": "True",
    },
    visibility = ["//visibility:private"],
)

selects.config_setting_group(
    name = "hermetic_enabled",
    match_any = [
        ":hermetic_explicitly_enabled",
        # Hermetic rust only works with hermetic cc.
        "//bazel/module_extensions/toolchains/rust:hermetic_enabled",
    ],
)

generate_cc_toolchains()

# Workaround for https://github.com/bazelbuild/bazel/issues/12712
cc_toolchain_suite(
    name = "cros-cc-toolchain-suite",
    toolchains = {
        "k8": ":x86_64-pc-linux-gnu",
        "x86_64": ":x86_64-pc-linux-gnu",
    },
)
