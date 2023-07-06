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
    files = depset(
        transitive = [
            target[BinaryPackageSetInfo].files
            for target in ctx.attr.deps
        ],
        order = "postorder",
    )

    return [
        DefaultInfo(files = files),
        BinaryPackageSetInfo(
            packages = packages,
            files = files,
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
