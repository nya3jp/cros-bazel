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

    http_file(
        name = "cros-sdk-2023.03.04.201551",
        sha256 = "2b5ba8a12806a1261c4b0a5e9e7f0b59c2a7922e828746daa01eee34dc31265c",
        urls = ["https://storage.googleapis.com/chromiumos-sdk/cros-sdk-2023.03.04.201551.tar.xz"]
    )
