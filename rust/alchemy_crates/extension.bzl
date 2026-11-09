# Copyright 2026 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Module extension providing @alchemy_crates from vendored definitions."""

load("@rules_rust//crate_universe/private:crates_vendor.bzl", "crates_vendor_remote_repository")
load("//bazel/rust/alchemy_crates/crates:defs.bzl", "crate_repositories")

def _alchemy_crates_impl(_module_ctx):
    crate_repositories()
    crates_vendor_remote_repository(
        name = "alchemy_crates",
        build_file = "//bazel/rust/alchemy_crates/crates:BUILD.bazel",
        defs_module = "//bazel/rust/alchemy_crates/crates:defs.bzl",
    )

alchemy_crates = module_extension(
    implementation = _alchemy_crates_impl,
)
