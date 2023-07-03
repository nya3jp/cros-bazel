# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@rules_rust//rust:toolchain.bzl", "rust_toolchain")
load("//bazel/platforms:platforms.bzl", "HOST_PLATFORM", "HOST_TRIPLE")

def _generate_rust_toolchain(name, platform_info, target_settings, package):
    def target(name):
        return "{package}:{triple}_{name}".format(
            package = package,
            triple = platform_info.triple,
            name = name,
        )

    rust_toolchain(
        name = name,
        binary_ext = "",
        cargo = target("cargo"),
        default_edition = "2021",
        dylib_ext = ".so",
        os = "linux",
        rust_doc = target("rustdoc"),
        rustfmt = target("rustfmt"),
        rust_std = target("rust_stdlibs"),
        rustc = target("rustc"),
        rustc_lib = target("rustc_libs"),
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
        name = "{}_toolchain".format(name),
        exec_compatible_with = [
            "@platforms//cpu:x86_64",
            "@platforms//os:linux",
        ],
        target_compatible_with = platform_info.constraints,
        toolchain = ":" + name,
        toolchain_type = "@rules_rust//rust:toolchain",
    )

def generate_rust_toolchains():
    _generate_rust_toolchain(
        name = "primordial",
        platform_info = HOST_PLATFORM,
        package = "//bazel/module_extensions/toolchains/files/primordial",
        target_settings = [
            ":hermetic_enabled",
            "//bazel/module_extensions/toolchains:primordial_enabled",
        ],
    )

    # TODO: Switch to ALL_PLATFORMS once it works.
    for platform_info in []:
        _generate_rust_toolchain(
            name = "bootstrapped_" + platform_info.triple,
            platform_info = platform_info,
            target_settings = [
                ":hermetic_enabled",
                "//bazel/module_extensions/toolchains:primordial_disabled",
            ],
            package = "//bazel/module_extensions/toolchains/files/bootstrapped",
        )
