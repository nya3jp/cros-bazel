# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# AUTO GENERATED DO NOT EDIT!
# Regenerate this file using the following command:
# bazel run @@//bazel/portage/bin/extract_package_from_manifest:sysroot_update
# However, you should never need to run this unless
# bazel explicitly tells you to.

# These three lines ensures that the following json is valid skylark.
false = False
true = True
null = None

SYSROOT_MANIFEST_CONTENT = {
    "root_package": {
        "name": "demo/shared_libs",
        "slot": "0/0",
    },
    "packages": [
        {
            "name": "demo/executable",
            "slot": "0/0",
            "content": {
                "/usr/bin/hello_world.sh": {},
            },
        },
        {
            "name": "demo/shared_libs",
            "slot": "0/0",
            "content": {
                "/lib32/libbaz.so.1.2.3": {},
                "/lib32/libfoo.so.1.2.3": {},
                "/lib64/libbar.so.1.2.3": {},
                "/lib64/libfoo.so": {
                    "symlink": true,
                },
                "/lib64/libfoo.so.1.2.3": {},
            },
        },
        {
            "name": "demo/symlinks",
            "slot": "0/0",
            "content": {
                "/path/to/hello.txt": {},
                "/symlinks/absolute_symlink.txt": {
                    "symlink": true,
                },
                "/symlinks/broken_absolute_symlink.txt": {
                    "symlink": true,
                },
                "/symlinks/broken_relative_symlink.txt": {
                    "symlink": true,
                },
                "/symlinks/relative_symlink.txt": {
                    "symlink": true,
                },
            },
        },
        {
            "name": "demo/sysroot",
            "slot": "0/0",
            "content": {},
        },
        {
            "name": "demo/system_headers",
            "slot": "0/0",
            "content": {
                "/usr/include/foo.h": {
                    "type": "HeaderFile",
                },
                "/usr/include/subdir/bar.h": {
                    "type": "HeaderFile",
                },
            },
        },
    ],
    "header_file_dirs": [
        "/usr/include",
    ],
    "header_file_dir_regexes": [
        "/usr/include",
    ],
}
