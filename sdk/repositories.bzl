# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

def cros_sdk(name, sha, extension="xz"):
    http_archive(
        name = name,
        sha256 = sha,
        urls = ["https://commondatastorage.googleapis.com/chromiumos-sdk/{}.tar.{}".format(name, extension)],

        build_file_content = """
exports_files(["root"])
""",

        # Switch to using `add_prefix` once we use Bazel 6.0 .
        # add_prefix="root",
        patch_cmds = [
            "mkdir .root",
            "find . -mindepth 1 -maxdepth 1 ! \\( -name 'WORKSPACE' -or -name 'BUILD.bazel' -or -name .root \\) -exec mv -- '{}' .root/ \\;",
            "mv .root root"
        ]
    )

def cros_sdk_repositories():
    cros_sdk("cros-sdk-2022.08.22.085953", "acc090ab670aca9a05b97412c0616e5674c2835dc18d3f205ad4ea3afeef90cf")
    cros_sdk("cros-sdk-2022.10.03.154923", "e900f5f12dbe29abd38b7499a831b643c9cce0f3453aabba06300f3bc20d6ca6")
