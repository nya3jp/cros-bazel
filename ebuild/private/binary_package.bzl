# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "BinaryPackageInfo")

def _binary_package_impl(ctx):
    src = ctx.file.src
    # TODO: Consider target/host transitions.
    transitive_runtime_deps_files = depset(
        [src],
        transitive = [dep[BinaryPackageInfo].transitive_runtime_deps_files for dep in ctx.attr.runtime_deps],
        order = "postorder",
    )

    transitive_runtime_deps_targets = depset(
        ctx.attr.runtime_deps,
        transitive = [dep[BinaryPackageInfo].transitive_runtime_deps_targets for dep in ctx.attr.runtime_deps],
        order = "postorder",
    )

    return [
        DefaultInfo(files = depset([src])),
        BinaryPackageInfo(
            file = src,
            transitive_runtime_deps_files = transitive_runtime_deps_files,
            transitive_runtime_deps_targets = transitive_runtime_deps_targets,
            direct_runtime_deps_targets = ctx.attr.runtime_deps,
        ),
    ]

binary_package = rule(
    implementation = _binary_package_impl,
    attrs = {
        "src": attr.label(
            mandatory = True,
            allow_single_file = [".tbz2"],
        ),
        "runtime_deps": attr.label_list(
            providers = [BinaryPackageInfo],
        ),
    },
)
