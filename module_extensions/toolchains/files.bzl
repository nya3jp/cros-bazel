# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@cros//bazel/platforms:platforms.bzl", "ALL_PLATFORMS")

PLATFORM_INDEPENDENT_FILES = {

    # TODO: minify this. At the moment, every action depends on the whole
    #  sysroot.
    "all_files": [":sysroot_files"],
    "ar": ["bin/llvm-ar"],
    "ar_files": [":sysroot_files"],
    "cargo": ["usr/bin/cargo"],
    # TODO(b/285460206): Switch to a platform-dependent clang.
    # For some reason when you use it screws with the include paths.
    "clang": ["usr/bin/clang"],
    "clang_cpp": ["usr/bin/clang++"],
    # See github.com/bazelbuild/bazel/issues/11122.
    "clang_selector": ["usr/bin/clang_selector"],
    "compiler_files": [":sysroot_files"],
    "dwp": ["bin/llvm-dwp"],
    "dwp_files": [":sysroot_files"],
    # A platform-dependent linker is also available.
    "ld": ["bin/ld.lld"],
    "linker_files": [":sysroot_files"],
    "nm": ["bin/llvm-nm"],
    "objcopy": ["bin/llvm-objcopy"],
    "objcopy_files": [":sysroot_files"],
    "objdump": ["bin/llvm-objdump"],
    "rustc": ["usr/bin/rustc"],
    "rustdoc": ["usr/bin/rustdoc"],
    "rustfmt": ["usr/bin/rustfmt"],
    "strip": ["bin/llvm-strip"],
    "strip_files": [":sysroot_files"],

    # Using directories as srcs in bazel is bad practice, but we get around the
    # problems by separately declaring a sysroot_files target.
    "sysroot": ["."],
}

PLATFORM_DEPENDENT_FILES = {
    "cpp": ["bin/{triple_no_host}-cpp"],
    "gcov": ["bin/{triple_no_host}-gcov"],
}

_PER_PLATFORM_UNFORMATTED = {
    k: "@toolchain_sdk//:" + k
    for k in PLATFORM_INDEPENDENT_FILES
} | {
    k: "@toolchain_sdk//:{k}_{{triple}}".format(k = k)
    for k in PLATFORM_DEPENDENT_FILES
}

# Usage: PER_PLATFORM_TOOLS[triple][tool_name] -> Label
PER_PLATFORM_FILES = {
    platform_info.triple: {
        k: Label(v.format(triple = platform_info.triple))
        for k, v in _PER_PLATFORM_UNFORMATTED.items()
    }
    for platform_info in ALL_PLATFORMS
}
