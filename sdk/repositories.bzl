# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")

def cros_sdk_repositories():
    http_file(
        name = "cros-sdk-2022.08.22.085953",
        sha256 = "acc090ab670aca9a05b97412c0616e5674c2835dc18d3f205ad4ea3afeef90cf",
        urls = ["https://commondatastorage.googleapis.com/chromiumos-sdk/cros-sdk-2022.08.22.085953.tar.xz"]
    )
