# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@io_bazel_rules_go//go:def.bzl", "go_library")

go_library(
    name = "makechroot",
    srcs = [
        "binpkgs.go",
        "overlay.go",
    ],
    importpath = "cros.local/bazel/ebuild/private/common/makechroot",
    visibility = ["//visibility:public"],
    deps = [
        "//bazel/ebuild/private/common/fileutil",
        "//bazel/ebuild/private/common/portage/xpak",
    ],
)
