# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:http.bzl", _http_file = "http_file")
load("//bazel/prebuilts:repositories.bzl", "prebuilts_dependencies")
load("//bazel/sdk:repositories.bzl", "cros_sdk_repositories")
load("//bazel/module_extensions/private:hub_repo.bzl", "hub_repo")

def _files_impl(module_ctx):
    symlinks = {}

    # Collect all these files into a single repo so we don't have to declare
    # every repo in MODULE.bazel.
    def http_file(name, **kwargs):
        symlinks[name] = "@{}//file".format(name)
        _http_file(name = name, **kwargs)

    http_file(
        name = "dumb_init",
        executable = True,
        sha256 = "e874b55f3279ca41415d290c512a7ba9d08f98041b28ae7c2acb19a545f1c4df",
        urls = ["https://github.com/Yelp/dumb-init/releases/download/v1.2.5/dumb-init_1.2.5_x86_64"],
    )

    prebuilts_dependencies(http_file = http_file)
    cros_sdk_repositories(http_file = http_file)

    hub_repo(name = "files", symlinks = symlinks)

files = module_extension(
    implementation = _files_impl,
)
