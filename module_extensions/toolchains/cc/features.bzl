# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/cc:action_names.bzl", "ACTION_NAMES")
load(
    "@bazel_tools//tools/cpp:cc_toolchain_config_lib.bzl",
    "feature",
    "flag_group",
    "flag_set",
)
load("//bazel/platforms:platforms.bzl", "HOST_TRIPLE")

def get_features(triple):
    features = _FEATURES[:]
    if triple == HOST_TRIPLE:
        features.extend(_HOST_FEATURES)
    return features

_HOST_FEATURES = []

# Taken from cs/chromeos_public/src/third_party/chromiumos-overlay/eclass/cros-bazel.eclass
# TODO(b/285459767): Revisit these features.
_FEATURES = [
    feature(
        name = "c_compiler",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = [
                            "--force-c-compiler",
                        ],
                    ),
                ],
            ),
        ],
    ),
    feature(name = "supports_pic", enabled = True),
    feature(
        name = "determinism",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.c_compile, ACTION_NAMES.cpp_compile],
                flag_groups = [
                    flag_group(
                        flags = [
                            # Make C++ compilation deterministic. Use linkstamping instead of these
                            # compiler symbols.
                            "-Wno-builtin-macro-redefined",
                            "-D__DATE__=\"redacted\"",
                            "-D__TIMESTAMP__=\"redacted\"",
                            "-D__TIME__=\"redacted\"",
                        ],
                    ),
                ],
            ),
        ],
    ),
    feature(
        name = "hardening",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.c_compile, ACTION_NAMES.cpp_compile],
                flag_groups = [
                    flag_group(
                        flags = [
                            # Conservative choice; -D_FORTIFY_SOURCE=2 may be unsafe in some cases.
                            # We need to undef it before redefining it as some distributions now
                            # have it enabled by default.
                            "-U_FORTIFY_SOURCE",
                            "-D_FORTIFY_SOURCE=1",
                            "-fstack-protector",
                        ],
                    ),
                ],
            ),
            flag_set(
                actions = [
                    ACTION_NAMES.cpp_link_dynamic_library,
                    ACTION_NAMES.cpp_link_nodeps_dynamic_library,
                ],
                flag_groups = [flag_group(flags = ["-Wl,-z,relro,-z,now"])],
            ),
            flag_set(
                actions = [
                    ACTION_NAMES.cpp_link_executable,
                ],
                flag_groups = [flag_group(flags = [
                    # TODO: This is enabled in the original file. Investigate
                    # if it should be enabled and why clang complains about
                    # "unused argument during compilation".
                    # "-pie",
                    "-Wl,-z,relro,-z,now",
                ])],
            ),
        ],
    ),
    feature(
        name = "warnings",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.c_compile, ACTION_NAMES.cpp_compile],
                flag_groups = [
                    flag_group(
                        flags = [
                            # All warnings are enabled. Maybe enable -Werror as well?
                            "-Wall",
                            # Add another warning that is not part of -Wall.
                            "-Wunused-but-set-parameter",
                            # But disable some that are problematic.
                            "-Wno-free-nonheap-object",  # has false positives
                        ],
                    ),
                ],
            ),
        ],
    ),
    feature(
        name = "no-canonical-prefixes",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_link_dynamic_library,
                    ACTION_NAMES.cpp_link_nodeps_dynamic_library,
                    ACTION_NAMES.cpp_link_executable,
                    ACTION_NAMES.preprocess_assemble,
                ],
                flag_groups = [flag_group(flags = ["-no-canonical-prefixes"])],
            ),
        ],
    ),
    feature(
        name = "linker-bin-path",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.cpp_link_dynamic_library,
                    ACTION_NAMES.cpp_link_nodeps_dynamic_library,
                    ACTION_NAMES.cpp_link_executable,
                ],
                flag_groups = [flag_group(flags = [
                    # Note: this differs from cros-bazel.eclass.
                    "--ld-path=%{sysroot}/bin/ld.lld",
                ])],
            ),
        ],
    ),
    feature(
        name = "disable-assertions",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.c_compile, ACTION_NAMES.cpp_compile],
                flag_groups = [flag_group(flags = ["-DNDEBUG"])],
            ),
        ],
    ),
    feature(
        name = "common",
        implies = [
            "determinism",
            "hardening",
            "warnings",
            "no-canonical-prefixes",
            "linker-bin-path",
        ],
    ),
    feature(
        name = "opt",
        implies = ["common", "disable-assertions"],
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.c_compile, ACTION_NAMES.cpp_compile],
                flag_groups = [
                    flag_group(
                        flags = ["-g0", "-O2", "-ffunction-sections", "-fdata-sections"],
                    ),
                ],
            ),
            flag_set(
                actions = [
                    ACTION_NAMES.cpp_link_dynamic_library,
                    ACTION_NAMES.cpp_link_nodeps_dynamic_library,
                    ACTION_NAMES.cpp_link_executable,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["-Wl,--gc-sections"],
                    ),
                ],
            ),
        ],
    ),
    feature(
        name = "fastbuild",
        implies = ["common"],
    ),
    feature(
        name = "dbg",
        implies = ["common"],
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.c_compile, ACTION_NAMES.cpp_compile],
                flag_groups = [
                    flag_group(
                        flags = ["-g"],
                    ),
                ],
            ),
        ],
    ),
]
