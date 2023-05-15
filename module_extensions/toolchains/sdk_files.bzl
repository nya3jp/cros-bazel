# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@cros//bazel/platforms:platforms.bzl", "ALL_PLATFORMS", "HOST_PLATFORM")
load("@rules_rust//rust:toolchain.bzl", "rust_stdlib_filegroup")

def generate_sdk_files():
    native.alias(
        name = "ar",
        actual = "bin/llvm-ar",
    )

    native.alias(
        name = "cargo",
        actual = "usr/bin/cargo",
    )

    native.alias(
        name = "clang",
        actual = "usr/bin/clang",
    )

    native.alias(
        name = "clang_cpp",
        actual = "usr/bin/clang++",
    )

    native.alias(
        name = "lld",
        actual = "bin/ld.lld",
    )

    native.alias(
        name = "rustc",
        actual = "usr/bin/rustc",
    )
    native.alias(
        name = "rustdoc",
        actual = "usr/bin/rustdoc",
    )

    native.alias(
        name = "rustfmt",
        actual = "usr/bin/rustfmt",
    )

    native.alias(
        name = "strip",
        actual = "bin/llvm-strip",
    )

    # TODO: implement platform-specific sysroots.
    native.filegroup(
        name = "sysroot_files",
        srcs = native.glob(["**"], exclude = [
            "BUILD.bazel",
            "WORKSPACE",
        ]),
    )

    # Using directories as srcs in bazel is bad practice, but we get around the
    # problems by sepererately declaring a sysroot_files target.
    native.filegroup(
        name = "sysroot",
        srcs = ["."],
    )

    for platform_info in ALL_PLATFORMS:
        _generate_files(platform_info)

def _generate_files(platform_info):
    triple = platform_info.triple

    rust_stdlib_filegroup(
        name = "{triple}_rust_stdlibs".format(triple = triple),
        srcs = native.glob([
            "usr/lib64/rustlib/{triple}/lib/*.rlib*".format(triple = triple),
            "usr/lib64/rustlib/{triple}/lib/*.a*".format(triple = triple),
            "usr/lib64/rustlib/{triple}/lib/*.so*".format(triple = triple),
        ]),
    )

    native.filegroup(
        name = "{triple}_rustc_libs".format(triple = triple),
        srcs = native.glob([
            # May need to consider other platforms.
            "usr/lib64/rustlib/{triple}/lib/*.so*".format(triple = triple),
            "usr/x86_64-cros-linux-gnu/lib64/*.so*",
            "usr/x86_64-cros-linux-gnu/usr/lib64/*.so*",
            "usr/lib64/*.so",
        ]),
    )

    native.filegroup(
        name = "{triple}_compiler_files".format(triple = triple),
        srcs = [
            ":sysroot_files".format(triple = triple),
        ],
    )
