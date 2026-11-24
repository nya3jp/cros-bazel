# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def cros_sdk_repositories(http_file):
    http_file(
        name = "cros-sdk",
        sha256 = "075874da5736e1129679431c0cdcde0d802bc081a10ece7f0f4f807f1ca670ed",
        urls = ["https://storage.googleapis.com/chromiumos-sdk/cros-sdk-2026.11.23.88972.tar.zst"],
        downloaded_file_path = "sdk.tar.zst",
    )

    http_file(
        name = "cros-bazel-sdk",
        sha256 = "fdd242a81296072c83ceb8089ee92882d8e57b600c8c01651447c350416a7d9a",
        urls = ["https://storage.googleapis.com/chromiumos-sdk/cros-bazel-sdk-2024.01.08.tar.zst"],
        downloaded_file_path = "cros-bazel-sdk.tar.zst",
    )
