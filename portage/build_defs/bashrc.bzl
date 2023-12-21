# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@rules_pkg//pkg:providers.bzl", "PackageArtifactInfo")
load("common.bzl", "BashrcInfo")

def _bashrc_impl(ctx):
    return [
        BashrcInfo(
            path = ctx.attr.path,
            layer = ctx.attr.layer,
        ),
        DefaultInfo(files = ctx.attr.layer[DefaultInfo].files),
    ]

bashrc = rule(
    implementation = _bashrc_impl,
    attrs = {
        "layer": attr.label(
            mandatory = True,
            providers = [PackageArtifactInfo],
            doc = """
            File: A file which represents an bashrc layer. A layer file can be
            a tar file (.tar or .tar.zst).
        """,
        ),
        "path": attr.string(
            mandatory = True,
            doc = """
            String: Path inside the container where the bashrc is mounted.
        """,
        ),
    },
)
