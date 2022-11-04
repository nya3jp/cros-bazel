# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "EbuildSrcInfo", "relative_path_in_label")

def _ebuild_tar_src_impl(ctx):
    return [
        DefaultInfo(files = depset([ctx.file.src])),
        EbuildSrcInfo(file = ctx.file.src, mount_path = ctx.attr.mount_path),
    ]

ebuild_tar_src = rule(
    implementation = _ebuild_tar_src_impl,
    attrs = {
        "src": attr.label(
            allow_single_file = ["tar.zst"],
            mandatory = True,
        ),
        "mount_path": attr.string(
            mandatory = True,
        ),
    },
)
