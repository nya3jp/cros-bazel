# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "BinaryPackageInfo")

def _binary_package_impl(ctx):
    src = ctx.file.src
    # TODO: Consider target/host transitions.
    runtime_deps = depset(
        [src],
        transitive = [dep[BinaryPackageInfo].runtime_deps for dep in ctx.attr.runtime_deps],
        order = "postorder",
    )
    return [
        DefaultInfo(files = depset([src])),
        BinaryPackageInfo(
            file = src,
            runtime_deps = runtime_deps,
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
