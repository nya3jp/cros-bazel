# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@rules_rust_non_bzlmod//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

def rust_toolchains():
    rules_rust_dependencies()

    rust_register_toolchains(
        edition = "2021",
    )
