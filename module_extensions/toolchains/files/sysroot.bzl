# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Rules for working with sysroots."""

load("@bazel_skylib//rules/directory:glob.bzl", "directory_glob")
load("@rules_cc//cc/toolchains:tool.bzl", "cc_tool")

visibility("public")

# Temporary rule. This will be integrated into rules_cc later.
def sysroot_prebuilt_binary(*, name, sysroot, exe, elf_file = None, **kwargs):
    wrapper_name = "_%s_wrapper" % name
    directory_glob(
        name = wrapper_name,
        directory = sysroot,
        srcs = [exe],
        data = [elf_file or exe + ".elf"],
    )

    cc_tool(
        name = name,
        src = wrapper_name,
        data = ["//bazel/module_extensions/toolchains/files:runtime"],
        **kwargs
    )
