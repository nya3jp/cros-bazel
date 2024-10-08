# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:git.bzl", "new_git_repository")

def depot_tools_repository():
    # TODO(b/322317375): Stop fetching the git repository and use the one at src/chromium/depot_tools.
    new_git_repository(
        name = "depot_tools",
        remote = "https://chromium.googlesource.com/chromium/tools/depot_tools.git",
        commit = "86752e9a55281200715749d75a88cf57bf2e7b01",
        shallow_since = "1703067784 +0000",
        patch_cmds = [
            # Force the cipd binaries and python venv to be downloaded.
            "DEPOT_TOOLS_UPDATE=0 ./ensure_bootstrap",
        ],
        build_file = "@//bazel/module_extensions/cros_deps:depot_tools/BUILD.depot_tools-template",
    )
