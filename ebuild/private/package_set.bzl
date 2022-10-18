# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "BinaryPackageInfo")

def _package_set_impl(ctx):
    transitive_runtime_deps_files = depset(
        transitive = [dep[BinaryPackageInfo].transitive_runtime_deps_files for dep in ctx.attr.deps],
        order = "postorder",
    )

    transitive_runtime_deps_targets = depset(
        ctx.attr.deps,
        transitive = [dep[BinaryPackageInfo].transitive_runtime_deps_targets for dep in ctx.attr.deps],
        order = "postorder",
    )

    return [
        DefaultInfo(files = transitive_runtime_deps_files),
        BinaryPackageInfo(
            file = None,
            transitive_runtime_deps_files = transitive_runtime_deps_files,
            transitive_runtime_deps_targets = transitive_runtime_deps_targets,
            direct_runtime_deps_targets = ctx.attr.deps,
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
