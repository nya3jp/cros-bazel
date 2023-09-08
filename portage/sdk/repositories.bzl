# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def cros_sdk_repositories(http_file):
    http_file(
        name = "cros-sdk-2023.09.08.050046",
        sha256 = "85617c5f49ac206d161927a7bf034ed28ee5bb9b01f7452ebb63d570b32f71f6",
        urls = ["https://storage.googleapis.com/chromiumos-sdk/cros-sdk-2023.09.08.050046.tar.xz"],
        downloaded_file_path = "sdk.tar.xz",
    )
