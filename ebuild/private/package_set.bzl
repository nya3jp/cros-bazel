# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "BinaryPackageInfo")

def _package_set_impl(ctx):
    transitive_install_files = depset(
        transitive = [dep[BinaryPackageInfo].transitive_install_files for dep in ctx.attr.deps],
        order = "postorder",
    )

    return [
        DefaultInfo(files = transitive_install_files),
    ]

package_set = rule(
    implementation = _package_set_impl,
    attrs = {
        "deps": attr.label_list(
            providers = [BinaryPackageInfo],
        ),
    },
)
