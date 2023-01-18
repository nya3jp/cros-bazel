# Copyright 2023 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@rules_rust//cargo:defs.bzl", "cargo_bootstrap_repository")
# buildifier: disable=bzl-visibility
load("@rules_rust//rust/private:common.bzl", "rust_common")
load("//bazel/ebuild/private/alchemist:src.bzl", "ALCHEMIST_SRCS")


def alchemist_repositories():
    cargo_bootstrap_repository(
        name = "alchemist",
        srcs = ALCHEMIST_SRCS,
        binary = "alchemist",
        cargo_lockfile = "//:Cargo.lock",
        cargo_toml = "//bazel/ebuild/private/alchemist:Cargo.toml",
        version = rust_common.default_version,
    )
