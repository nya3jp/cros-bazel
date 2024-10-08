# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/module_extensions/private:hub_repo.bzl", "hub_init")
load("//bazel/portage/common/testdata:prebuilts.bzl", "portage_testdata_prebuilts")
load("//bazel/portage/sdk:repositories.bzl", "cros_sdk_repositories")

def _files_impl(module_ctx):
    hub = hub_init()

    hub.http_file.alias_and_symlink(
        name = "dumb_init",
        executable = True,
        sha256 = "e874b55f3279ca41415d290c512a7ba9d08f98041b28ae7c2acb19a545f1c4df",
        urls = ["https://github.com/Yelp/dumb-init/releases/download/v1.2.5/dumb-init_1.2.5_x86_64"],
    )

    # Statically-linked bash.
    # It is used by alchemist to evaluate ebuilds, and in some unit tests.
    hub.http_file.alias_and_symlink(
        name = "bash-static",
        downloaded_file_path = "bash",
        executable = True,
        sha256 = "64469a9512a00199c85622ec56f870f97d50457a4e06e0cfa39bae7adf0cc8f2",
        urls = ["https://github.com/robxu9/bash-static/releases/download/5.2.015-1.2.3-2/bash-linux-x86_64"],
    )

    hub.http_archive.alias_and_symlink(
        name = "patchelf",
        sha256 = "ce84f2447fb7a8679e58bc54a20dc2b01b37b5802e12c57eece772a6f14bf3f0",
        urls = ["https://github.com/NixOS/patchelf/releases/download/0.18.0/patchelf-0.18.0-x86_64.tar.gz"],
        build_file_content = "exports_files(['bin/patchelf'])",
        targets = "//:bin/patchelf",
    )

    hub.http_file.alias_and_symlink(
        name = "buildozer",
        urls = ["https://github.com/bazelbuild/buildtools/releases/download/v6.1.2/buildozer-linux-amd64"],
        executable = True,
        sha256 = "2aef0f1ef80a0140b8fe6e6a8eb822e14827d8855bfc6681532c7530339ea23b",
    )

    hub.gs_file.alias_only(
        name = "remotetool",
        url = "gs://chromeos-localmirror/cros-bazel/remotetool-20240102",
        executable = True,
    )

    cros_sdk_repositories(http_file = hub.http_file.alias_only)

    portage_testdata_prebuilts(
        prebuilt_binpkg = hub.prebuilt_binpkg.alias_and_symlink,
    )

    hub.generate_hub_repo(name = "files")

files = module_extension(
    implementation = _files_impl,
)
