# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/portage/build_defs:common.bzl", "BinaryPackageInfo", "get_all_deps")

visibility("public")

def _collect_ebuild_metadata_impl(target, ctx):
    depsets = []

    direct = []
    if BinaryPackageInfo in target:
        direct.append(target[BinaryPackageInfo].metadata)

    for dep in get_all_deps(ctx):
        if OutputGroupInfo in dep:
            output_groups = dep[OutputGroupInfo]
            d = getattr(output_groups, "ebuild_metadata", None)
            if d:
                depsets.append(d)

    return [
        OutputGroupInfo(ebuild_metadata = depset(direct, transitive = depsets)),
    ]

collect_ebuild_metadata_aspect = aspect(
    implementation = _collect_ebuild_metadata_impl,
    attr_aspects = ["*"],
    attrs = dict(),
)
