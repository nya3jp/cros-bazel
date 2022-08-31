# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@io_bazel_rules_go//go:def.bzl", "go_library", "go_test")

go_library(
    name = "dependency",
    srcs = [
        "algorithm_base.go",
        "algorithm_extra.go",
        "atom.go",
        "expr.go",
        "expr_compile.go",
    ],
    importpath = "cros.local/bazel/ebuild/private/common/standard/dependency",
    visibility = ["//visibility:public"],
    deps = [
        "//bazel/ebuild/private/common/standard/dependency/internal/grammer",
        "//bazel/ebuild/private/common/standard/naming",
        "//bazel/ebuild/private/common/standard/version",
    ],
)

go_test(
    name = "dependency_test",
    srcs = ["atom_test.go"],
    deps = [
        ":dependency",
        "//bazel/ebuild/private/common/standard/version",
    ],
)
