# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def cros_sdk_repositories(http_file):
    http_file(
        name = "cros-sdk-2023.04.05.144808",
        sha256 = "2c5a36ffd06d8a6afaaff35da08b922215a7eb222e053e4aba4f83fd1dce5a58",
        urls = ["https://storage.googleapis.com/chromiumos-sdk/cros-sdk-2023.04.05.144808.tar.xz"],
        downloaded_file_path = "sdk.tar.xz",
    )

    http_file(
        name = "cros-sdk-2023.08.08.170046",
        sha256 = "e11afdd9fd80a1aba30dce678a65720c89c967291774feb14f7ec3559a7bd666",
        urls = ["https://storage.googleapis.com/chromiumos-sdk/cros-sdk-2023.08.08.170046.tar.xz"],
        downloaded_file_path = "sdk.tar.xz",
    )
