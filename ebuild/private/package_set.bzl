# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "BinaryPackageInfo")

def _package_set_impl(ctx):
    deps = depset(
        transitive = [dep[BinaryPackageInfo].runtime_deps for dep in ctx.attr.deps],
        order = "postorder",
    )
    return [DefaultInfo(files = deps)]

package_set = rule(
    implementation = _package_set_impl,
    attrs = {
        "deps": attr.label_list(
            providers = [BinaryPackageInfo],
        ),
    },
)
