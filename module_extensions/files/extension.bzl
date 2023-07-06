# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:http.bzl", _http_file_symlink = "http_file")
load("//bazel/module_extensions/private:hub_repo.bzl", "hub_repo")
load("//bazel/portage/repo_defs/prebuilts:repositories.bzl", "prebuilts_dependencies")
load("//bazel/portage/sdk:repositories.bzl", "cros_sdk_repositories")

def _files_impl(module_ctx):
    aliases = {}
    symlinks = {}

    # Collect all these files into a single repo so we don't have to declare
    # every repo in MODULE.bazel.
    def http_file_symlink(name, **kwargs):
        symlinks[name] = "@{}//file".format(name)
        _http_file_symlink(name = name, **kwargs)

    def http_file_alias(name, **kwargs):
        aliases[name] = "@{}//file".format(name)
        _http_file_symlink(name = name, **kwargs)

    http_file_symlink(
        name = "dumb_init",
        executable = True,
        sha256 = "e874b55f3279ca41415d290c512a7ba9d08f98041b28ae7c2acb19a545f1c4df",
        urls = ["https://github.com/Yelp/dumb-init/releases/download/v1.2.5/dumb-init_1.2.5_x86_64"],
    )

    # Statically-linked bash.
    # It is used by alchemist to evaluate ebuilds, and in some unit tests.
    http_file_symlink(
        name = "bash-static",
        downloaded_file_path = "bash",
        executable = True,
        sha256 = "64469a9512a00199c85622ec56f870f97d50457a4e06e0cfa39bae7adf0cc8f2",
        urls = ["https://github.com/robxu9/bash-static/releases/download/5.2.015-1.2.3-2/bash-linux-x86_64"],
    )

    prebuilts_dependencies(http_file = http_file_alias)
    cros_sdk_repositories(http_file = http_file_alias)

    hub_repo(name = "files", aliases = aliases, symlinks = symlinks)

files = module_extension(
    implementation = _files_impl,
)
