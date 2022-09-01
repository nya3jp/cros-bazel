# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@io_bazel_rules_go//go:def.bzl", "go_library")

go_library(
    name = "packages",
    srcs = [
        "package.go",
        "stability.go",
    ],
    importpath = "cros.local/bazel/ebuild/private/common/standard/packages",
    visibility = ["//visibility:public"],
    deps = [
        "//bazel/ebuild/private/common/standard/dependency",
        "//bazel/ebuild/private/common/standard/makevars",
        "//bazel/ebuild/private/common/standard/version",
    ],
)
