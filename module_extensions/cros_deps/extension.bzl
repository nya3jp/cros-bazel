# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load(":depot_tools/repositories.bzl", "depot_tools_repository")
load(":chromite/repositories.bzl", "chromite")
load("//bazel/zstd:repositories.bzl", "zstd_repository")

def _cros_deps_impl(module_ctx):
    chromite(name = "chromite")
    depot_tools_repository()
    zstd_repository()

cros_deps = module_extension(
    implementation = _cros_deps_impl,
)
