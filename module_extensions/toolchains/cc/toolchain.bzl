# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load(
    "@bazel_tools//tools/cpp:cc_toolchain_config_lib.bzl",
    "tool",
    "tool_path",
)
load("@rules_cc//cc:defs.bzl", "cc_toolchain", "cc_toolchain_suite")
load("//bazel/module_extensions/toolchains:files.bzl", "PER_PLATFORM_FILES")
load("//bazel/platforms:platforms.bzl", "ALL_PLATFORMS")
load(":features.bzl", "get_features")

SYSROOT = "external/_main~toolchains~toolchain_sdk"
EXTRA_INCLUDES = [
    "/usr/include/c++/v1",
    "/usr/lib64/clang/16/include",
]

def _cc_toolchain_config_impl(ctx):
    def tool(name, file):
        # The path is external/<repo rule>/<path relative to sysroot>.
        return tool_path(name = name, path = file.path.split("/", 2)[2])

    tool_paths = [
        tool(name = "ar", file = ctx.file.ar),
        tool(name = "compat-ld", file = ctx.file.ld),
        tool(name = "cpp", file = ctx.file.cpp),
        tool(name = "dwp", file = ctx.file.dwp),
        tool(name = "gcc", file = ctx.file.gcc),
        tool(name = "gcov", file = ctx.file.gcov),
        tool(name = "ld", file = ctx.file.ld),
        tool(name = "nm", file = ctx.file.nm),
        tool(name = "objcopy", file = ctx.file.objcopy),
        tool(name = "objdump", file = ctx.file.objdump),
        tool(name = "strip", file = ctx.file.strip),
    ]

    sysroot_path = ctx.file.sysroot.path

    # Documented at
    # https://docs.bazel.build/versions/main/skylark/lib/cc_common.html#create_cc_toolchain_config_info.
    #
    # create_cc_toolchain_config_info is the public interface for registering
    # C++ toolchain behavior.
    return cc_common.create_cc_toolchain_config_info(
        ctx = ctx,
        features = get_features(ctx.attr.triple),
        tool_paths = tool_paths,
        toolchain_identifier = "{triple}-cc-toolchain".format(triple = ctx.attr.triple),
        compiler = "clang",
        builtin_sysroot = SYSROOT,
        cxx_builtin_include_directories = ["%sysroot%" + include for include in EXTRA_INCLUDES],
        # These fields are only used for toolchain resolution when using the
        # old crosstool_top. They're not used when using platforms.
        target_system_name = "",
        target_cpu = "",
        target_libc = "",
    )

cc_toolchain_config = rule(
    implementation = _cc_toolchain_config_impl,
    attrs = dict(
        triple = attr.string(mandatory = True),
        sysroot = attr.label(mandatory = True, allow_single_file = True),
        ar = attr.label(mandatory = True, allow_single_file = True),
        cpp = attr.label(mandatory = True, allow_single_file = True),
        dwp = attr.label(mandatory = True, allow_single_file = True),
        gcc = attr.label(mandatory = True, allow_single_file = True),
        gcov = attr.label(mandatory = True, allow_single_file = True),
        ld = attr.label(mandatory = True, allow_single_file = True),
        nm = attr.label(mandatory = True, allow_single_file = True),
        objcopy = attr.label(mandatory = True, allow_single_file = True),
        objdump = attr.label(mandatory = True, allow_single_file = True),
        strip = attr.label(mandatory = True, allow_single_file = True),
    ),
    provides = [CcToolchainConfigInfo],
)

def generate_cc_toolchain(name, platform):
    triple = platform.triple
    files = PER_PLATFORM_FILES[triple]
    config_name = "config_{name}".format(name = name)

    cc_toolchain_config(
        name = config_name,
        triple = triple,
        sysroot = files["sysroot"],
        ar = files["ar"],
        cpp = files["cpp"],
        dwp = files["dwp"],
        gcc = files["clang_selector"],
        gcov = files["gcov"],
        ld = files["ld"],
        nm = files["nm"],
        objcopy = files["objcopy"],
        objdump = files["objdump"],
        strip = files["strip"],
    )

    cc_toolchain(
        name = name,
        all_files = files["all_files"],
        ar_files = files["ar_files"],
        compiler_files = files["compiler_files"],
        dwp_files = files["dwp_files"],
        linker_files = files["linker_files"],
        objcopy_files = files["objcopy_files"],
        strip_files = files["strip_files"],
        toolchain_config = ":" + config_name,
    )

    native.toolchain(
        name = "native_{name}".format(name = name),
        exec_compatible_with = [
            "@platforms//cpu:x86_64",
            "@platforms//os:linux",
        ],
        target_compatible_with = platform.constraints,
        toolchain = ":" + name,
        target_settings = [
            "@@//bazel/module_extensions/toolchains/cc:hermetic_enabled",
        ],
        toolchain_type = "@bazel_tools//tools/cpp:toolchain_type",
    )

def generate_cc_toolchains():
    for platform in ALL_PLATFORMS:
        generate_cc_toolchain(
            name = "cc_toolchain_{triple}".format(triple = platform.triple),
            platform = platform,
        )

    # Workaround for https://github.com/bazelbuild/bazel/issues/12712
    cc_toolchain_suite(
        name = "cc-toolchain-suite",
        toolchains = {
            "k8": ":cc_toolchain_x86_64-pc-linux-gnu",
            "x86_64": ":cc_toolchain_x86_64-pc-linux-gnu",
        },
    )
