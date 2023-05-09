# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# Definitions for high-level information about the toolchains we support.

# The fields of each instance are ordered as they are to read more naturally for
# folks used to reading triples. Please do not sort the ones that land in a
# triple alphabetically. :)
ToolchainDesc = provider(
    "Represents all of the general information we need about a native toolchain.",
    fields = {
        "cpu_arch": "The name of the CPU architecture within Chrome OS.",
        "vendor": "The vendor of the target platform; this is 'pc' for host " +
                  "toolchains, and 'cros' for target toolchains.",
        "abi": "The ABI this architecture uses. This is often `gnu`.",
    },
)

# Host toolchain.
host_toolchain_desc = ToolchainDesc(
    cpu_arch = "x86_64",
    vendor = "pc",
    abi = "gnu",
)

# TODO(gbiv): desc_to_triple and bazel_cpu_arch should probably be turned into
# autopopulated fields on `ToolchainDesc`. This can be done with `provider`'s
# `init` field, which is apparently quite new and not included in our current
# bazel version (bazel-5).
def bazel_cpu_arch(desc):
    """Returns the @platforms cpu label of the given `desc`."""
    arch_name = "armv7" if desc.cpu_arch == "armv7a" else desc.cpu_arch
    return "@platforms//cpu:" + arch_name

def desc_to_triple(desc):
    """Turns a toolchain description into a triple string."""
    return "{}-{}-linux-{}".format(desc.cpu_arch, desc.vendor, desc.abi)

all_toolchain_descs = [
    host_toolchain_desc,

    # Target x86_64 toolchain
    ToolchainDesc(
        cpu_arch = "x86_64",
        vendor = "cros",
        abi = "gnu",
    ),

    # Target ARMv7a toolchain
    ToolchainDesc(
        cpu_arch = "armv7a",
        vendor = "cros",
        abi = "gnueabihf",
    ),

    # Target aarch64 toolchain
    ToolchainDesc(
        cpu_arch = "aarch64",
        vendor = "cros",
        abi = "gnu",
    ),
]
