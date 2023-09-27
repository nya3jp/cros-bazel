# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/portage/build_defs:test_helpers.bzl", "assert_eq", "assert_not_none")
load(":files.bzl", "ELF_BINARY", "HEADER", "SHARED_LIBRARY", "SYMLINK", "UNKNOWN", "get_extracted_files", "get_file_type", "preprocess_content", "resolve")

visibility("private")

_SHARED_LIB_PATH = "/lib64/libc.so.6"
_SYMLINK_PATH = "/lib64/libc.so"
_INTERP_PATH = "/lib64/ld-linux-x86-64.so.2"
_BINARY_PATH = "/usr/bin/nano"
_DIR_SYMLINK = "/lib"
_FILES = {
    _SHARED_LIB_PATH: {"type": SHARED_LIBRARY},
    _SYMLINK_PATH: {
        "target": _SHARED_LIB_PATH,
        "type": SYMLINK,
    },
    _INTERP_PATH: {"type": SHARED_LIBRARY},
    _BINARY_PATH: {
        "libs": {"libc.so.6": _SYMLINK_PATH},
        "type": ELF_BINARY,
    },
    _DIR_SYMLINK: {
        "target": "/lib64",
        "type": SYMLINK,
    },
}

def _test_files_impl(ctx):
    files = {}

    ordered_files = [path for (path, _) in preprocess_content(_FILES)]
    assert_eq(ordered_files, [
        "/lib",
        "/lib64/ld-linux-x86-64.so.2",
        "/lib64/libc.so.6",
        "/lib64/libc.so",
        "/usr/bin/nano",
    ])

    executable = "sysroot"
    get_extracted_files(ctx, files = files, content = _FILES)

    shared_lib = files[_SHARED_LIB_PATH]
    assert_eq(shared_lib.runfiles.to_list(), [shared_lib.file])

    symlink = files[_SYMLINK_PATH]
    assert_eq(resolve(symlink), shared_lib, "Want symlink to point to:\n%r\n\nGot:\n%r")
    assert_eq(sorted(symlink.runfiles.to_list()), [symlink.file, shared_lib.file])

    dir_symlink = files[_DIR_SYMLINK]
    assert_not_none(get_file_type(dir_symlink, UNKNOWN))

    interp = files[_INTERP_PATH]
    bin = files[_BINARY_PATH]
    bin_info = get_file_type(bin, ELF_BINARY)
    assert_not_none(bin_info)
    assert_eq(bin_info.interp, interp)
    assert_eq(bin_info.libs.to_list(), [symlink])

    assert_eq(get_file_type(shared_lib, HEADER), None)
    assert_eq(get_file_type(symlink, HEADER), None)

    assert_not_none(get_file_type(shared_lib, SHARED_LIBRARY))
    assert_not_none(get_file_type(symlink, SHARED_LIBRARY))
    assert_eq(get_file_type(symlink, SHARED_LIBRARY, follow_symlinks = False), None)

    runfiles = sorted(bin.runfiles.to_list())
    assert_eq(runfiles.pop().basename, "nano.elf")
    assert_eq(runfiles, [interp.file, symlink.file, shared_lib.file, bin.file])

    # Ensure bazel doesn't complain about no action generating these files
    runfiles = depset(transitive = [f.runfiles for f in files.values()])
    for f in runfiles.to_list():
        ctx.actions.write(f, "")

test_files = rule(
    implementation = _test_files_impl,
)
