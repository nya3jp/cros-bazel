# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@io_bazel_rules_go//go:def.bzl", "go_binary", "go_library")

go_library(
    name = "build_sdk_lib",
    srcs = ["main.go"],
    embedsrcs = ["setup.sh"],
    importpath = "cros.local/bazel/ebuild/private/cmd/build_sdk",
    visibility = ["//visibility:private"],
    deps = [
        "//bazel/ebuild/private/common/fileutil",
        "//bazel/ebuild/private/common/makechroot",
        "//bazel/ebuild/private/common/runfiles",
        "@com_github_urfave_cli//:cli",
        "@io_bazel_rules_go//go/tools/bazel:go_default_library",
    ],
)

go_binary(
    name = "build_sdk",
    data = [
        "//bazel/ebuild/private/cmd/run_in_container",
    ],
    embed = [":build_sdk_lib"],
    visibility = ["//visibility:public"],
)
