# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""cros_toolchains is a module extension for importing the toolchains from the CrOS SDK."""

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
load("//bazel/module_extensions/toolchains:extract_sdk.bzl", "extract_sdk")

def _toolchains_impl(module_ctx):
    # Bazel's http_archive appears to modify the .elf and .so files, producing
    # different outputs to extracting the http_file directly.
    http_file(
        name = "toolchain_sdk_tarball",
        url = "https://storage.googleapis.com/chromiumos-sdk/2024/05/x86_64-cros-linux-gnu-2024.05.25.66621.tar.xz",
        integrity = "sha256-raqmmpbuPeIKdeyWbjlSXGfBikTwlDH1iKMWlKl6pe0=",
    )

    extract_sdk(
        name = "toolchain_sdk",
        tarball = "@toolchain_sdk_tarball//file:downloaded",
    )

toolchains = module_extension(
    implementation = _toolchains_impl,
)
