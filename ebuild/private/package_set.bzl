# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "BinaryPackageInfo")

def _package_set_impl(ctx):
    direct_runtime_deps = [
        target[BinaryPackageInfo]
        for target in ctx.attr.deps
    ]
    transitive_runtime_deps = depset(
        direct_runtime_deps,
        transitive = [
            pkg.transitive_runtime_deps
            for pkg in direct_runtime_deps
        ],
        order = "postorder",
    )
    all_files = depset(
        transitive = [pkg.all_files for pkg in direct_runtime_deps],
    )

    return [
        DefaultInfo(files = all_files),
        # TODO: Do not return BinaryPackageInfo from package_set. It is suitable
        # for a single binary package, but not for a set of it.
        BinaryPackageInfo(
            file = None,
            all_files = all_files,
            direct_runtime_deps = direct_runtime_deps,
            transitive_runtime_deps = transitive_runtime_deps,
        ),
    ]

package_set = rule(
    implementation = _package_set_impl,
    attrs = {
        "deps": attr.label_list(
            providers = [BinaryPackageInfo],
        ),
    },
)
