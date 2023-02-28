# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")

def cros_sdk_repositories():
    http_file(
        name = "cros-sdk-2022.08.25.153812",
        sha256 = "05f2aa50cc1c3bad7c6fb0de61176ff86cc3f7c75fb289c29ea3954bf409b9e5",
        urls = ["https://commondatastorage.googleapis.com/chromiumos-sdk/cros-sdk-2022.08.25.153812.tar.xz"]
    )
