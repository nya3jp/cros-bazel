# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/portage/repo_defs/alpine:repositories.bzl", "alpine_repository")
load("//bazel/portage/repo_defs/zstd:repositories.bzl", "zstd_repository")
load(":chromite/repositories.bzl", "chromite")
load(":depot_tools/repositories.bzl", "depot_tools_repository")

def _cros_deps_impl(_module_ctx):
    alpine_repository()
    chromite(name = "chromite")
    depot_tools_repository()
    zstd_repository()

cros_deps = module_extension(
    implementation = _cros_deps_impl,
)
