# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "BinaryPackageInfo")

def _binary_package_impl(ctx):
    src = ctx.file.src
    direct_runtime_deps = [
        target[BinaryPackageInfo] for target in ctx.attr.runtime_deps
    ]
    transitive_runtime_deps = depset(
        direct_runtime_deps,
        transitive = [pkg.transitive_runtime_deps for pkg in direct_runtime_deps],
        order = "postorder",
    )
    all_files = depset(
        [src],
        transitive = [pkg.all_files for pkg in direct_runtime_deps],
    )
    return [
        DefaultInfo(files = depset([src])),
        BinaryPackageInfo(
            file = src,
            all_files = all_files,
            direct_runtime_deps = direct_runtime_deps,
            transitive_runtime_deps = transitive_runtime_deps,
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
