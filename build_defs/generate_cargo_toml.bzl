# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Generates Cargo.toml files from dependencies specified in the build file."""

load("//bazel/build_defs/generate_cargo_toml:generate_cargo_toml.bzl", _generate_cargo_toml = "generate_cargo_toml")

visibility("public")

generate_cargo_toml = _generate_cargo_toml
