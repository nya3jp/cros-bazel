# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@io_bazel_rules_go//go:def.bzl", "go_binary", "go_library")

go_library(
    name = "extract_deps_lib",
    srcs = ["main.go"],
    importpath = "cros.local/bazel/ebuild/private/cmd/extract_deps",
    visibility = ["//visibility:private"],
    deps = [
        "//bazel/ebuild/private/common/portage/makeconf",
        "//bazel/ebuild/private/common/portage/portagevars",
        "//bazel/ebuild/private/common/portage/repository",
        "//bazel/ebuild/private/common/runfiles",
        "//bazel/ebuild/private/common/standard/dependency",
        "//bazel/ebuild/private/common/standard/ebuild",
        "//bazel/ebuild/private/common/standard/packages",
        "@com_github_urfave_cli//:cli",
    ],
)

go_binary(
    name = "extract_deps",
    embed = [":extract_deps_lib"],
    visibility = ["//visibility:public"],
)
