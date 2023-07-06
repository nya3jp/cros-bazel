# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""cros_toolchains is a module extension for importing the toolchains from the CrOS SDK."""

load(
    "//bazel/rust/module_extension:repo.bzl",
    "RUST_ATTRS",
    "rust_repo",
)

def _cros_toolchains_repo_impl(repo_ctx):
    rust_repo(repo_ctx)

_cros_toolchains_repo = repository_rule(
    implementation = _cros_toolchains_repo_impl,
    local = True,
    attrs = RUST_ATTRS,
)

def _cros_toolchains_impl(module_ctx):
    _cros_toolchains_repo(name = "cros_toolchains")

cros_toolchains = module_extension(
    implementation = _cros_toolchains_impl,
)
