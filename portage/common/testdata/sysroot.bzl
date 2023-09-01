# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# AUTO GENERATED DO NOT EDIT!
# Regenerate this file using the following command:
# bazel run @@//bazel/portage/bin/extract_package_from_manifest:sysroot_update
# However, you should never need to run this unless
# bazel explicitly tells you to.

SYSROOT_MANIFEST_CONTENT = {
    "root_package": {
        "name": "demo/shared_libs",
        "slot": "0/0",
    },
    "packages": [
        {
            "name": "demo/executable",
            "slot": "0/0",
            "symlinks": [],
            "files": [
                "/usr/bin/hello_world.sh",
            ],
            "header_files": [],
            "shared_libraries": [],
        },
        {
            "name": "demo/shared_libs",
            "slot": "0/0",
            "symlinks": [
                "/lib64/libfoo.so",
            ],
            "files": [
                "/lib32/libbaz.so.1.2.3",
                "/lib32/libfoo.so.1.2.3",
                "/lib64/libbar.so.1.2.3",
                "/lib64/libfoo.so.1.2.3",
            ],
            "header_files": [],
            "shared_libraries": [
                "/lib32/libbaz.so.1.2.3",
                "/lib64/libbar.so.1.2.3",
                "/lib64/libfoo.so",
                "/lib64/libfoo.so.1.2.3",
            ],
        },
        {
            "name": "demo/symlinks",
            "slot": "0/0",
            "symlinks": [
                "/symlinks/absolute_symlink.txt",
                "/symlinks/broken_absolute_symlink.txt",
                "/symlinks/broken_relative_symlink.txt",
                "/symlinks/relative_symlink.txt",
            ],
            "files": [
                "/path/to/hello.txt",
            ],
            "header_files": [],
            "shared_libraries": [],
        },
        {
            "name": "demo/sysroot",
            "slot": "0/0",
            "symlinks": [],
            "files": [],
            "header_files": [],
            "shared_libraries": [],
        },
        {
            "name": "demo/system_headers",
            "slot": "0/0",
            "symlinks": [],
            "files": [
                "/usr/include/foo.h",
                "/usr/include/subdir/bar.h",
            ],
            "header_files": [
                "/usr/include/foo.h",
                "/usr/include/subdir/bar.h",
            ],
            "shared_libraries": [],
        },
    ],
    "header_file_dirs": [
        "/usr/include",
    ],
}
