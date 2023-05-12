# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@cros//bazel/module_extensions/toolchains:platforms.bzl", "all_toolchain_descs", "host_toolchain_desc")
load("@rules_rust//rust:toolchain.bzl", "rust_stdlib_filegroup")

def generate_sdk_files():
    native.alias(
        name = "cargo",
        actual = "usr/bin/cargo",
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
        name = "lld",
        actual = "bin/ld.lld",
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

    for desc in all_toolchain_descs:
        _generate_files(desc)

def _generate_files(desc):
    triple = desc.triple

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

    native.alias(
        name = "{triple}_clang".format(triple = triple),
        actual = "usr/bin/{triple}-clang".format(triple = triple),
    )

    native.alias(
        name = "{triple}_clang++".format(triple = triple),
        actual = "usr/bin/{triple}-clang++".format(triple = triple),
    )

    native.alias(
        name = "{triple}_strip".format(triple = triple),
        actual = "usr/bin/{triple}-strip".format(triple = triple),
    )

    native.filegroup(
        name = "{triple}_compiler_files".format(triple = triple),
        srcs = [
            ":{triple}_clang".format(triple = triple),
            ":{triple}_clang++".format(triple = triple),
            ":sysroot_files".format(triple = triple),
        ],
    )
