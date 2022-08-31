# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/ebuild:defs.bzl", "overlay_set")

overlay_set(
    name = "overlays",
    overlays = [
        "//overlays/overlay-arm64-generic",
        "//third_party/eclass-overlay",
        "//third_party/chromiumos-overlay",
        "//third_party/portage-stable",
    ],
    visibility = ["//visibility:public"],
)
