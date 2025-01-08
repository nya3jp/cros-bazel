# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def cros_sdk_repositories(http_file):
    http_file(
        name = "cros-sdk",
        sha256 = "f1687d0ad62ccc6d8d647528b6abca123ed02a2414a2d9280da518e61d3a2b81",
        urls = ["https://storage.googleapis.com/chromiumos-sdk/cros-sdk-2025.01.08.73853.tar.zst"],
        downloaded_file_path = "sdk.tar.zst",
    )

    http_file(
        name = "cros-bazel-sdk",
        sha256 = "fdd242a81296072c83ceb8089ee92882d8e57b600c8c01651447c350416a7d9a",
        urls = ["https://storage.googleapis.com/chromiumos-sdk/cros-bazel-sdk-2024.01.08.tar.zst"],
        downloaded_file_path = "cros-bazel-sdk.tar.zst",
    )
