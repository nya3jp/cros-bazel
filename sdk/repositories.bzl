# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("utils.bzl", "cros_sdk_repository")

def cros_sdk_repositories():
    cros_sdk_repository(
        name = "cros-sdk-2022.08.22.085953",
        sha256 = "acc090ab670aca9a05b97412c0616e5674c2835dc18d3f205ad4ea3afeef90cf",
    )
    cros_sdk_repository(
        name = "cros-sdk-2022.10.03.154923",
        sha256 = "e900f5f12dbe29abd38b7499a831b643c9cce0f3453aabba06300f3bc20d6ca6",
    )
