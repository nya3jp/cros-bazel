# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@rules_rust//rust:toolchain.bzl", "rust_toolchain")
load("//bazel/platforms:platforms.bzl", "ALL_PLATFORMS", "HOST_TRIPLE")

def _generate_rust_toolchain(platform_info):
    toolchain_name = platform_info.triple
    rust_toolchain(
        name = toolchain_name,
        binary_ext = "",
        cargo = "@toolchain_sdk//:cargo",
        default_edition = "2021",
        dylib_ext = ".so",
        os = "linux",
        rust_doc = "@toolchain_sdk//:rustdoc",
        rustfmt = "@toolchain_sdk//:rustfmt",
        rust_std = "@toolchain_sdk//:rust_stdlibs_{}".format(platform_info.triple),
        rustc = "@toolchain_sdk//:rustc",
        rustc_lib = "@toolchain_sdk//:rustc_libs_{}".format(platform_info.triple),
        staticlib_ext = ".a",
        stdlib_linkflags = [
            "-lpthread",
            "-ldl",
            "-lc++",
        ],
        target_triple = platform_info.triple,
        exec_triple = HOST_TRIPLE,
    )

    native.toolchain(
        name = "{}_toolchain".format(toolchain_name),
        exec_compatible_with = [
            "@platforms//cpu:x86_64",
            "@platforms//os:linux",
        ],
        target_compatible_with = platform_info.constraints,
        toolchain = ":" + toolchain_name,
        target_settings = [":hermetic_enabled"],
        toolchain_type = "@rules_rust//rust:toolchain",
    )

def generate_rust_toolchains():
    for platform_info in ALL_PLATFORMS:
        _generate_rust_toolchain(platform_info)
