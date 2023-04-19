# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "OverlaySetInfo")
load("@rules_pkg//pkg:providers.bzl", "PackageArtifactInfo")

def _overlay_set_impl(ctx):
    return [
        OverlaySetInfo(
            layers = [pkg[PackageArtifactInfo].file for pkg in ctx.attr.overlays],
        ),
    ]

overlay_set = rule(
    implementation = _overlay_set_impl,
    attrs = {
        "overlays": attr.label_list(
            providers = [PackageArtifactInfo],
        ),
    },
)
