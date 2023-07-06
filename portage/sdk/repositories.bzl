# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def cros_sdk_repositories(http_file):
    http_file(
        name = "cros-sdk-2022.08.25.153812",
        sha256 = "05f2aa50cc1c3bad7c6fb0de61176ff86cc3f7c75fb289c29ea3954bf409b9e5",
        urls = ["https://commondatastorage.googleapis.com/chromiumos-sdk/cros-sdk-2022.08.25.153812.tar.xz"],
        downloaded_file_path = "sdk.tar.xz",
    )

    http_file(
        name = "cros-sdk-2023.04.05.144808",
        sha256 = "2c5a36ffd06d8a6afaaff35da08b922215a7eb222e053e4aba4f83fd1dce5a58",
        urls = ["https://storage.googleapis.com/chromiumos-sdk/cros-sdk-2023.04.05.144808.tar.xz"],
        downloaded_file_path = "sdk.tar.xz",
    )
