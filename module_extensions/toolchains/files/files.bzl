# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

visibility("//bazel/module_extensions/toolchains/files/...")

TOOLS = {
    "ar": "bin/llvm-ar",
    "cargo": "usr/bin/cargo",
    # TODO(b/285460206): Switch to a platform-dependent clang.
    # For some reason when you use it screws with the include paths.
    "clang": "usr/bin/clang",
    "clang_cpp": "usr/bin/clang++",
    "clang_selector": "usr/bin/clang_selector",
    "dwp": "bin/llvm-dwp",
    # A platform-dependent linker is also available.
    "ld": "bin/ld.lld",
    "nm": "bin/llvm-nm",
    "objcopy": "bin/llvm-objcopy",
    "objdump": "bin/llvm-objdump",
    "rustc": "usr/bin/rustc",
    "rustdoc": "usr/bin/rustdoc",
    "rustfmt": "usr/bin/rustfmt",
    "strip": "bin/llvm-strip",
    "cpp": "bin/{triple_no_host}-cpp",
    "gcov": "bin/{triple_no_host}-gcov",
}

TOOLCHAIN_FILEGROUPS = {
    "all_files": ["**"],
    "ar_files": ["**"],
    "compiler_files": ["**"],
    "dwp_files": ["**"],
    "linker_files": ["**"],
    "objcopy_files": ["**"],
    "strip_files": ["**"],
    "runtime_files": [
        "lib/*",
        "lib64/ld-linux-x86-64.so.2",
    ],
}

FILEGROUPS = TOOLCHAIN_FILEGROUPS | {
    k: [v]
    for k, v in TOOLS.items()
}
