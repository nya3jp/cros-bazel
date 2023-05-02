# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//cros/toolchain:platforms.bzl", "all_toolchain_descs", "bazel_cpu_arch", "desc_to_triple")

EmuInfo = provider(
    fields = ["emulator_path"],
)

def _impl(ctx):
    toolchain_info = platform_common.ToolchainInfo(
        emuinfo = EmuInfo(
            emulator_path = ctx.attr.emulator_path,
        ),
    )
    return [toolchain_info]

emulation_toolchain = rule(
    implementation = _impl,
    attrs = {
        "emulator_path": attr.string(),
    },
    provides = [platform_common.ToolchainInfo],
)

def _generate_emulation_toolchain(desc):
    triple = desc_to_triple(desc)

    if desc.vendor == "pc":
        emulator_path = "/lib64/ld-linux-x86-64.so.2"
    else:
        cpu_arch = desc.cpu_arch
        if cpu_arch == "armv7a":
            cpu_arch = "arm"
        emulator_path = "qemu-" + cpu_arch

    impl_name = "cros-sdk-{}-emulation_impl".format(triple)
    emulation_toolchain(
        name = impl_name,
        emulator_path = emulator_path,
    )

    native.toolchain(
        name = impl_name[:-len("_impl")],
        exec_compatible_with = [
            "@platforms//cpu:x86_64",
            "@platforms//os:linux",
        ],
        target_compatible_with = [
            bazel_cpu_arch(desc),
            "@platforms//os:linux",
            "//cros/platforms/vendor:" + desc.vendor,
        ],
        toolchain = ":" + impl_name,
        toolchain_type = ":toolchain_type",
    )

def generate_emulation_toolchains():
    for desc in all_toolchain_descs:
        _generate_emulation_toolchain(desc)
