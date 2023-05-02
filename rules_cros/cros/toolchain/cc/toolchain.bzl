# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/cc:action_names.bzl", "ACTION_NAMES")
load(
    "@bazel_tools//tools/cpp:cc_toolchain_config_lib.bzl",
    "action_config",
    "flag_group",
    "flag_set",
    "tool",
)
load("//cros/toolchain:platforms.bzl", "all_toolchain_descs", "bazel_cpu_arch", "desc_to_triple")

_DEFAULT_LINKER_FLAGS = [
    flag_set(
        flag_groups = [
            flag_group(
                flags = [
                    "-Wl,-znow",
                    "-Wl,-zrelro",
                    "-Wl,--no-rosegment",
                    "-Wl,--hash-style=gnu",
                ],
            ),
        ],
    ),
]

def _impl(ctx):
    vendor = ctx.attr.vendor
    is_host = vendor != "cros"

    arch = ctx.attr.cpu_arch
    abi = ctx.attr.abi
    triple = "{}-{}-linux-{}".format(arch, vendor, abi)
    usr_bin_triple = "/usr/bin/" + triple + "-"
    action_configs = [
        action_config(
            action_name = ACTION_NAMES.cpp_compile,
            tools = [
                tool(
                    path = usr_bin_triple + "clang++",
                ),
            ],
        ),
        action_config(
            action_name = ACTION_NAMES.c_compile,
            tools = [
                tool(
                    path = usr_bin_triple + "clang",
                ),
            ],
        ),
        action_config(
            action_name = ACTION_NAMES.cpp_link_executable,
            tools = [
                tool(
                    path = usr_bin_triple + "clang++",
                ),
            ],
            flag_sets = _DEFAULT_LINKER_FLAGS,
        ),
        action_config(
            action_name = ACTION_NAMES.cpp_link_dynamic_library,
            tools = [
                tool(
                    path = usr_bin_triple + "clang++",
                ),
            ],
            flag_sets = _DEFAULT_LINKER_FLAGS,
        ),
        action_config(
            action_name = ACTION_NAMES.cpp_link_nodeps_dynamic_library,
            tools = [
                tool(
                    path = usr_bin_triple + "clang++",
                ),
            ],
            flag_sets = _DEFAULT_LINKER_FLAGS,
        ),
        action_config(
            action_name = ACTION_NAMES.cpp_link_static_library,
            tools = [
                tool(
                    path = usr_bin_triple + "clang++",
                ),
            ],
            flag_sets = _DEFAULT_LINKER_FLAGS,
        ),
        action_config(
            action_name = ACTION_NAMES.strip,
            tools = [
                tool(
                    path = usr_bin_triple + "strip",
                ),
            ],
        ),
    ]

    clang_version = "15.0.0"
    gcc_version = "10.2.0"

    # FIXME(gbiv): Autodetect clang/gcc versions.
    if is_host:
        sysroot = None
        include_dirs = [
            "/usr/include/c++/v1",
            "/usr/lib64/clang/{}/include".format(clang_version),
            "/usr/local/include",
            "/include",
            "/usr/include",
        ]
    else:
        sysroot = "/usr/" + triple
        include_dirs = [
            "/usr/lib64/clang/" + clang_version,
            "/usr/lib64/gcc/{}/{}".format(triple, gcc_version),
            "%sysroot%/usr/local/include",
            "%sysroot%/include",
            "%sysroot%/usr/include",
        ]

    # Documented at
    # https://docs.bazel.build/versions/main/skylark/lib/cc_common.html#create_cc_toolchain_config_info.
    #
    # create_cc_toolchain_config_info is the public interface for registering
    # C++ toolchain behavior.
    return cc_common.create_cc_toolchain_config_info(
        ctx = ctx,
        toolchain_identifier = "cros-sdk-" + triple,
        host_system_name = "linux",
        target_system_name = "local",
        target_cpu = arch,
        target_libc = "glibc-2.33",
        compiler = "clang",
        abi_version = "local",
        abi_libc_version = "local",
        action_configs = action_configs,
        builtin_sysroot = sysroot,
        cxx_builtin_include_directories = include_dirs,
    )

cc_toolchain_config = rule(
    implementation = _impl,
    attrs = {
        "abi": attr.string(mandatory = True),
        "cpu_arch": attr.string(mandatory = True),
        "vendor": attr.string(mandatory = True),
    },
    provides = [CcToolchainConfigInfo],
)

def _generate_cc_toolchain(desc):
    if desc.vendor == "pc":
        config_name = "host_toolchain_config"
    else:
        config_name = desc.cpu_arch + "_board_toolchain_config"

    triple = desc_to_triple(desc)
    cc_toolchain_config(name = config_name, abi = desc.abi, cpu_arch = desc.cpu_arch, vendor = desc.vendor)
    cc_toolchain_name = "cros-sdk-{}-cc".format(triple)
    native.cc_toolchain(
        name = cc_toolchain_name,
        all_files = ":none",
        compiler_files = ":none",
        dwp_files = ":none",
        linker_files = ":none",
        objcopy_files = ":none",
        strip_files = ":none",
        toolchain_config = ":" + config_name,
    )

    native.toolchain(
        name = "cros-sdk-{}-cc-toolchain".format(triple),
        exec_compatible_with = [
            "@platforms//cpu:x86_64",
            "@platforms//os:linux",
        ],
        target_compatible_with = [
            bazel_cpu_arch(desc),
            "@platforms//os:linux",
            "//cros/platforms/vendor:" + desc.vendor,
        ],
        toolchain = ":" + cc_toolchain_name,
        toolchain_type = "@bazel_tools//tools/cpp:toolchain_type",
    )

def generate_cc_toolchains():
    for desc in all_toolchain_descs:
        _generate_cc_toolchain(desc)
