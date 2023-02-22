# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "OverlaySetInfo")
load("@rules_pkg//pkg:mappings.bzl", "pkg_files", "strip_prefix")
load("@rules_pkg//pkg:tar.bzl", "pkg_tar")
load("@rules_pkg//pkg:providers.bzl", "PackageArtifactInfo")

def overlay(name, srcs, mount_path, **kwargs):
    files_target = name + "_tarfiles"
    pkg_files(
        name = files_target,
        srcs = srcs,
        prefix = 'mnt/host/source/' + mount_path,
        strip_prefix = strip_prefix.from_pkg(),
        visibility = ["//visibility:private"],
    )
    pkg_tar(
        name = name,
        extension = "tar.zst",
        compressor = "@//bazel/ebuild/private:zstd",
        compressor_args = "--threads=0",
        srcs = [files_target],
        **kwargs
    )

def _overlay_set_impl(ctx):
    return [
        OverlaySetInfo(
            overlays = [file[PackageArtifactInfo] for file in ctx.attr.overlays],
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
