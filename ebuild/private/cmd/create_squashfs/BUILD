# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@io_bazel_rules_go//go:def.bzl", "go_binary", "go_library")

go_library(
    name = "create_squashfs_lib",
    srcs = ["main.go"],
    importpath = "cros.local/bazel/ebuild/private/cmd/create_squashfs",
    visibility = ["//visibility:private"],
    deps = ["@com_github_urfave_cli//:cli"],
)

go_binary(
    name = "create_squashfs",
    embed = [":create_squashfs_lib"],
    visibility = ["//visibility:public"],
)
