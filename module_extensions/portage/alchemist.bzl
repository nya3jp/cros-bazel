# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@rules_rust//cargo:defs.bzl", "cargo_bootstrap_repository")
load("//bazel/portage/bin/alchemist:src.bzl", "ALCHEMIST_REPO_RULE_SRCS")

_BUILD_MODE = "release"
_TOOL_TEMPLATE = "@rust_host_tools//:bin/{tool}"

def alchemist(name):
    cargo_bootstrap_repository(
        name = name,
        srcs = ALCHEMIST_REPO_RULE_SRCS,
        binary = "alchemist",
        build_mode = _BUILD_MODE,
        cargo_lockfile = "//bazel/portage/bin/alchemist:Cargo.lock",
        cargo_toml = "//bazel/portage/bin/alchemist:Cargo.toml",
        rust_toolchain_cargo_template = "@@//bazel/module_extensions/portage:alchemist_cargo_wrapper.sh",
        rust_toolchain_rustc_template = _TOOL_TEMPLATE,
        # TODO(b/307348075) Remove once the Alchemist build is faster
        timeout = 1200,
    )
