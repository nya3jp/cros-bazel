# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def _platforminfo_init(*, name, cpu_arch, vendor, abi):
    kwargs = dict(name = name, cpu_arch = cpu_arch, vendor = vendor, abi = abi)
    triple = "{}-{}-linux-{}".format(cpu_arch, vendor, abi)
    arch_name = "armv7" if cpu_arch == "armv7a" else cpu_arch
    cpu_arch = Label("@platforms//cpu:" + arch_name)
    vendor = Label("//bazel/platforms/constraints:vendor_" + vendor)
    abi = Label("//bazel/platforms/constraints:abi_" + abi)
    os = Label("@platforms//os:linux")
    return dict(
        name = name,
        kwargs = kwargs,
        cpu = cpu_arch,
        vendor = vendor,
        abi = abi,
        os = os,
        triple = triple,
        constraints = [
            abi,
            cpu_arch,
            os,
            vendor,
        ],
    )

PlatformInfo, _new_platforminfo = provider(
    "Represents all of the general information we need about a native toolchain.",
    fields = {
        "name": "The human-readable name for the platform",
        "kwargs": "The kwargs used to originally construct the toolchain desc",
        "constraints": "The bazel constraints placed on the platform",
        "cpu": "The cpu constraint",
        "vendor": "The vendor constraint; this is ':vendor_pc' for host " +
                  "toolchains, and ':vendor_cros' for target toolchains.",
        "os": "The OS constraint.",
        "abi": "The ABI this architecture uses. This is often `gnu`.",
        "triple": "The platform's triple.",
    },
    init = _platforminfo_init,
)
