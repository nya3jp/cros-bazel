# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@io_bazel_rules_go//go:def.bzl", "go_library", "go_test")

go_library(
    name = "version",
    srcs = ["version.go"],
    importpath = "cros.local/bazel/ebuild/private/common/standard/version",
    visibility = ["//visibility:public"],
)

go_test(
    name = "version_test",
    srcs = ["version_test.go"],
    deps = [":version"],
)
