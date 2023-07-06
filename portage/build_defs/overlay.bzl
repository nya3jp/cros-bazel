# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "OverlayInfo", "OverlaySetInfo")
load("@rules_pkg//pkg:providers.bzl", "PackageArtifactInfo")

def _overlay_set_impl(ctx):
    return [
        OverlaySetInfo(
            layers = [overlay[OverlayInfo].layer[PackageArtifactInfo].file for overlay in ctx.attr.overlays],
        ),
    ]

overlay_set = rule(
    implementation = _overlay_set_impl,
    attrs = {
        "overlays": attr.label_list(
            providers = [OverlayInfo],
        ),
    },
)

def _overlay_impl(ctx):
    return [
        OverlayInfo(
            path = ctx.attr.path,
            layer = ctx.attr.layer,
        ),
    ]

overlay = rule(
    implementation = _overlay_impl,
    attrs = {
        "path": attr.string(
            mandatory = True,
            doc = """
            String: Path inside the container where the overlay's ebuilds are
            mounted.
        """,
        ),
        "layer": attr.label(
            mandatory = True,
            providers = [PackageArtifactInfo],
            doc = """
            File: A file which represents an overlay layer. A layer file can be
            a tar file (.tar or .tar.zst).
        """,
        ),
    },
)
