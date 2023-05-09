# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@rules_rust//rust:toolchain.bzl", "rust_toolchain")
load("//rules_cros/toolchains:platforms.bzl", "all_toolchain_descs", "bazel_cpu_arch", "desc_to_triple")

def _generate_rust_toolchain(desc):
    triple = desc_to_triple(desc)

    # TODO(b/215615637): Change this target triple back to x86_64-pc-linux-gnu
    if desc.vendor == "pc":
        custom_target_triple = triple.replace("-pc-", "-unknown-")
    else:
        custom_target_triple = triple

    toolchain_name = "cros-sdk-{}-rust_impl".format(triple)
    rust_toolchain(
        name = toolchain_name,
        binary_ext = "",
        cargo = "@cros_toolchains//rust/bin/x86_64-pc-linux-gnu:cargo",
        clippy_driver = "@cros_toolchains//rust/bin/x86_64-pc-linux-gnu:clippy-driver",
        default_edition = "2018",
        dylib_ext = ".so",
        os = "linux",
        rust_doc = "@cros_toolchains//rust/bin/x86_64-pc-linux-gnu:rustdoc",
        rust_std = "@cros_toolchains//rust/lib/{}:rust_stdlibs".format(triple),
        rustc = "@cros_toolchains//rust/bin/x86_64-pc-linux-gnu:rustc",
        rustc_lib = "@cros_toolchains//rust/lib/{}:rustc_libs".format(triple),
        rustfmt = "@cros_toolchains//rust/bin/x86_64-pc-linux-gnu:rustfmt",
        staticlib_ext = ".a",
        stdlib_linkflags = [
            "-lpthread",
            "-ldl",
            "-lc++",
        ],
        target_triple = custom_target_triple,
        exec_triple = "x86_64-unknown-linux-gnu",
    )

    native.toolchain(
        name = "cros-sdk-{}-rust".format(triple),
        exec_compatible_with = [
            "@platforms//cpu:x86_64",
            "@platforms//os:linux",
        ],
        target_compatible_with = [
            bazel_cpu_arch(desc),
            "@platforms//os:linux",
            "@cros//rules_cros/platforms/vendor:" + desc.vendor,
        ],
        toolchain = ":" + toolchain_name,
        toolchain_type = "@rules_rust//rust:toolchain",
    )

def generate_rust_toolchains():
    for desc in all_toolchain_descs:
        _generate_rust_toolchain(desc)
