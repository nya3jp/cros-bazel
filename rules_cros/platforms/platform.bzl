# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//cros/toolchain:platforms.bzl", "all_toolchain_descs", "bazel_cpu_arch", "desc_to_triple")

def define_platforms():
    for desc in all_toolchain_descs:
        if desc.vendor == "pc":
            platform_name = "linux"
        else:
            platform_name = "cros"

        native.platform(
            name = "{}_{}".format(platform_name, desc.cpu_arch),
            constraint_values = [
                bazel_cpu_arch(desc),
                "@platforms//os:linux",
                "//cros/platforms/vendor:" + desc.vendor,
            ],
        )
