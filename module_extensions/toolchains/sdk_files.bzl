# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@cros//bazel/platforms:platforms.bzl", "ALL_PLATFORMS", "HOST_PLATFORM")
load("@rules_rust//rust:toolchain.bzl", "rust_stdlib_filegroup")
load("//bazel/module_extensions/toolchains/cc:toolchain.bzl", "generate_cc_toolchains")
load(":files.bzl", "PLATFORM_DEPENDENT_FILES", "PLATFORM_INDEPENDENT_FILES")

def generate_sdk_files():
    for name, files in PLATFORM_INDEPENDENT_FILES.items():
        native.filegroup(
            name = name,
            srcs = files,
        )

    # TODO: implement platform-specific sysroots.
    native.filegroup(
        name = "sysroot_files",
        srcs = native.glob(["**"], exclude = [
            "BUILD.bazel",
            "WORKSPACE",
        ]),
    )

    native.filegroup(name = "libs", srcs = native.glob(["lib/*.so*"]))
    native.filegroup(name = "interp", srcs = ["lib64/ld-linux-x86-64.so.2"])

    for platform_info in ALL_PLATFORMS:
        _generate_files(platform_info)

    # Bazel's C++ toolchains require that relative paths are relative to the
    # directory the toolchain is defined in, not the execroot. This has the
    # annoying consesquence that the toolchains need to be defined here.
    generate_cc_toolchains()

def _generate_files(platform_info):
    triple = platform_info.triple

    # We don't have an SDK for a host toolchain, so we use the closest thing
    # available.
    if platform_info == HOST_PLATFORM:
        triple_no_host = "x86_64-cros-linux-gnu"
    else:
        triple_no_host = triple

    rust_stdlib_filegroup(
        name = "rust_stdlibs_{triple}".format(triple = triple),
        srcs = native.glob([
            "usr/lib64/rustlib/{triple}/lib/*.rlib*".format(triple = triple),
            "usr/lib64/rustlib/{triple}/lib/*.a*".format(triple = triple),
            "usr/lib64/rustlib/{triple}/lib/*.so*".format(triple = triple),
        ]),
    )

    native.filegroup(
        name = "rustc_libs_{triple}".format(triple = triple),
        srcs = native.glob([
            # May need to consider other platforms.
            "usr/lib64/rustlib/{triple}/lib/*.so*".format(triple = triple),
            "usr/x86_64-cros-linux-gnu/lib64/*.so*",
            "usr/x86_64-cros-linux-gnu/usr/lib64/*.so*",
            "usr/lib64/*.so",
        ]),
    )

    for name, files in PLATFORM_DEPENDENT_FILES.items():
        files = [f.format(triple = triple, triple_no_host = triple_no_host) for f in files]

        native.filegroup(
            name = "{name}_{triple}".format(name = name, triple = triple),
            srcs = files,
        )
