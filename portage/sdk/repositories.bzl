# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def cros_sdk_repositories(http_file):
    http_file(
        name = "cros-sdk",
        sha256 = "66012c7cf8ab97505286ad2310444e6282319785304256af2dcd514773647dde",
        urls = ["https://storage.googleapis.com/chromiumos-sdk/cros-sdk-2026.07.22.86269.tar.zst"],
        downloaded_file_path = "sdk.tar.zst",
    )

    http_file(
        name = "cros-bazel-sdk",
        sha256 = "fdd242a81296072c83ceb8089ee92882d8e57b600c8c01651447c350416a7d9a",
        urls = ["https://storage.googleapis.com/chromiumos-sdk/cros-bazel-sdk-2024.01.08.tar.zst"],
        downloaded_file_path = "cros-bazel-sdk.tar.zst",
    )
