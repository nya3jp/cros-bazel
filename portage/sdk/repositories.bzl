# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def cros_sdk_repositories(http_file):
    http_file(
        name = "cros-sdk",
        sha256 = "7e0d5765be16471c26f7f7712e4dee48bde403bd8d6af03f81b3cee4e15140c4",
        urls = ["https://storage.googleapis.com/chromiumos-sdk/cros-sdk-2023.11.15.020032.tar.xz"],
        downloaded_file_path = "sdk.tar.xz",
    )
