# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def cros_sdk_repositories(http_file):
    http_file(
        name = "cros-sdk",
        sha256 = "e72b0c1d2f88c6ee0f62046747dd9612a5ebff9c5871033579b4a78b79d0373b",
        urls = ["https://storage.googleapis.com/chromiumos-sdk/cros-sdk-2023.10.03.020014.tar.xz"],
        downloaded_file_path = "sdk.tar.xz",
    )
