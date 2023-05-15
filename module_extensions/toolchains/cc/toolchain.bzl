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
load("//bazel/platforms:platforms.bzl", "ALL_PLATFORMS")
load("@toolchain_sdk//:sysroot.bzl", "ABSOLUTE_SYSROOT_PATH")

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

def _cc_toolchain_config_impl(ctx):
    cpp_compile = tool(tool = ctx.file.cpp_compile)
    c_compile = tool(tool = ctx.file.c_compile)
    strip = tool(tool = ctx.file.strip)
    ar = tool(tool = ctx.file.ar)
    ld = tool(tool = ctx.file.ld)

    sysroot_path = ctx.file.sysroot.path

    linker_flags = _DEFAULT_LINKER_FLAGS + [
        flag_set(
            flag_groups = [
                flag_group(flags = [
                    "--ld-path=" + ctx.file.ld.path,
                    # TODO: Once go/cros-build:hermetic-bazel-binary-outputs is implemented, remove this.
                    "-static-libstdc++",
                ]),
            ],
        ),
    ]

    action_groups = [
        struct(
            actions = [
                ACTION_NAMES.cpp_compile,
                ACTION_NAMES.assemble,
                ACTION_NAMES.preprocess_assemble,
            ],
            kwargs = dict(tools = [cpp_compile], flag_sets = []),
        ),
        struct(
            actions = [
                ACTION_NAMES.c_compile,
            ],
            kwargs = dict(tools = [c_compile], flag_sets = []),
        ),
        struct(
            actions = [
                ACTION_NAMES.cpp_link_executable,
                ACTION_NAMES.cpp_link_dynamic_library,
                ACTION_NAMES.cpp_link_nodeps_dynamic_library,
            ],
            kwargs = dict(tools = [cpp_compile], flag_sets = linker_flags),
        ),
        struct(
            actions = [
                ACTION_NAMES.strip,
            ],
            kwargs = dict(tools = [strip], flag_sets = []),
        ),
        struct(
            actions = [
                ACTION_NAMES.objc_archive,
                ACTION_NAMES.cpp_link_static_library,
            ],
            kwargs = dict(tools = [ar], flag_sets = []),
        ),
    ]

    action_configs = []
    for action_group in action_groups:
        for action in action_group.actions:
            action_configs.append(action_config(
                action_name = action,
                **action_group.kwargs
            ))

    # Documented at
    # https://docs.bazel.build/versions/main/skylark/lib/cc_common.html#create_cc_toolchain_config_info.
    #
    # create_cc_toolchain_config_info is the public interface for registering
    # C++ toolchain behavior.
    return cc_common.create_cc_toolchain_config_info(
        ctx = ctx,
        toolchain_identifier = "{triple}-cc-toolchain".format(triple = ctx.attr.triple),
        compiler = "clang",
        action_configs = action_configs,
        builtin_sysroot = sysroot_path,
        cxx_builtin_include_directories = [
            # TODO: Absolute paths won't work with RBE, but relative paths won't work with clang (see github.com/bazelbuild/bazel/issues/4605). Ideally, we want to fix clang to work with relative paths, and then replace this with "%sysroot%/usr/include/c++/v1".
            ABSOLUTE_SYSROOT_PATH + "/usr/include/c++/v1",
            ABSOLUTE_SYSROOT_PATH + "/usr/lib64/clang/16/include",
        ],
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
        c_compile = attr.label(mandatory = True, allow_single_file = True),
        cpp_compile = attr.label(mandatory = True, allow_single_file = True),
        strip = attr.label(mandatory = True, allow_single_file = True),
        ld = attr.label(mandatory = True, allow_single_file = True),
        ar = attr.label(mandatory = True, allow_single_file = True),
    ),
    provides = [CcToolchainConfigInfo],
)

def _generate_cc_toolchain(platform_info):
    triple = platform_info.triple
    config_name = "{triple}_config".format(triple = triple)

    def label(name):
        return Label(("@toolchain_sdk//:" + name).format(triple = triple))

    cc_toolchain_config(
        name = config_name,
        triple = triple,
        sysroot = label("sysroot"),
        c_compile = label("clang"),
        cpp_compile = label("clang_cpp"),
        strip = label("strip"),
        ld = label("lld"),
        ar = label("ar"),
    )

    toolchain_name = triple
    native.cc_toolchain(
        name = triple,
        # TODO: can this be minified? Maybe sysroot works, which might symlink a single directory rather than thousands of files every time.
        all_files = label("sysroot_files"),
        ar_files = label("ar"),
        compiler_files = label("{triple}_compiler_files"),
        dwp_files = label("{triple}_compiler_files"),
        linker_files = label("{triple}_compiler_files"),
        objcopy_files = label("{triple}_compiler_files"),
        strip_files = label("strip"),
        toolchain_config = ":" + config_name,
    )

    native.toolchain(
        name = "{triple}_native".format(triple = triple),
        exec_compatible_with = [
            "@platforms//cpu:x86_64",
            "@platforms//os:linux",
            "//bazel/platforms/constraints:hermetic_cc_toolchain_enabled",
        ],
        target_compatible_with = platform_info.constraints,
        toolchain = ":" + toolchain_name,
        toolchain_type = "@bazel_tools//tools/cpp:toolchain_type",
    )

def generate_cc_toolchains():
    for platform_info in ALL_PLATFORMS:
        _generate_cc_toolchain(platform_info)
