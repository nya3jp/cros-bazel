# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# buildifier: disable=module-docstring
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")
load("//cros/toolchain:platforms.bzl", "all_toolchain_descs", "desc_to_triple")
load("//cros/toolchain/rust:rust_repository.bzl", "cros_rust_repository")

def rules_cros_dependencies():
    maybe(
        http_archive,
        name = "rules_rust",
        sha256 = "4d9a243e69a1e9d4f6538ea2c7f2cd8c811ddc6003aa01f16d4a2d69f65d2856",
        strip_prefix = "rules_rust-b188f1b1eb67e2a596c80c362f94b5218b388c7a",
        urls = [
            # Main branch as of 2022-01-19
            "https://github.com/bazelbuild/rules_rust/archive/b188f1b1eb67e2a596c80c362f94b5218b388c7a.tar.gz",
        ],
    )

    rules_cros_toolchains()

def rules_cros_toolchains(name = "cros_toolchains"):
    cros_rust_repository(name = name)

    toolchains = []
    for desc in all_toolchain_descs:
        triple = desc_to_triple(desc)
        toolchains += [
            "@rules_cros//cros/toolchain/cc:cros-sdk-{}-cc-toolchain".format(triple),
            "@rules_cros//cros/toolchain/rust:cros-sdk-{}-rust".format(triple),
            "@rules_cros//cros/toolchain/emulation:cros-sdk-{}-emulation".format(triple),
        ]

    native.register_toolchains(*toolchains)
