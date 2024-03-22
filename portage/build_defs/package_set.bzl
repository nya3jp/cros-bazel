# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "BinaryPackageSetInfo")

def _package_set_impl(ctx):
    packages = depset(
        transitive = [
            target[BinaryPackageSetInfo].packages
            for target in ctx.attr.deps
        ],
        order = "postorder",
    )
    partials = depset(
        transitive = [
            target[BinaryPackageSetInfo].partials
            for target in ctx.attr.deps
        ],
    )

    return [
        DefaultInfo(files = partials),
        BinaryPackageSetInfo(
            packages = packages,
            partials = partials,
        ),
    ]

package_set = rule(
    implementation = _package_set_impl,
    attrs = {
        "deps": attr.label_list(
            providers = [BinaryPackageSetInfo],
        ),
    },
)
