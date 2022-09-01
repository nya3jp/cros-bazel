# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/ebuild:defs.bzl", "binary_package")

# fakeroot-1.27.tbz2 is a binary package generated with:
# https://chromium-review.googlesource.com/c/chromiumos/overlays/portage-stable/+/3827925
binary_package(
    name = "fakeroot",
    src = "fakeroot-1.27.tbz2",
    # TODO: Enable this runtime dependency.
    # It's currently disabled as it's already installed in SDK.
    # runtime_deps = ["//third_party/portage-stable/libcap"],
    visibility = ["//visibility:public"],
)
