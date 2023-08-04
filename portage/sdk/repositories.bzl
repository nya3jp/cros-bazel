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
        name = "cros-sdk-2023.08.03.170038",
        sha256 = "3e938046e11f57f6964da10ceb5d67aa72ac66ff25e690ba3d2baa2a285eacfa",
        urls = ["https://storage.googleapis.com/chromiumos-sdk/cros-sdk-2023.08.03.170038.tar.xz"],
        downloaded_file_path = "sdk.tar.xz",
    )
