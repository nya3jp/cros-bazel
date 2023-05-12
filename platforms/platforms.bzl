# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load(":platform_provider.bzl", "PlatformInfo")

HOST_PLATFORM = PlatformInfo(
    name = "host",
    cpu_arch = "x86_64",
    vendor = "pc",
    abi = "gnu",
)

HOST_TRIPLE = HOST_PLATFORM.triple

TARGET_PLATFORMS = [
    # Target x86_64 toolchain
    PlatformInfo(
        name = "amd64-generic",
        cpu_arch = "x86_64",
        vendor = "cros",
        abi = "gnu",
    ),

    # Target ARMv7a toolchain
    PlatformInfo(
        name = "arm32-generic",
        cpu_arch = "armv7a",
        vendor = "cros",
        abi = "gnueabihf",
    ),

    # Target aarch64 toolchain
    PlatformInfo(
        name = "arm64-generic",
        cpu_arch = "aarch64",
        vendor = "cros",
        abi = "gnu",
    ),
]

ALL_PLATFORMS = [HOST_PLATFORM] + TARGET_PLATFORMS
