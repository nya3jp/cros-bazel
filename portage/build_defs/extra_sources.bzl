# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@rules_pkg//pkg:mappings.bzl", "pkg_attributes", "pkg_files", "strip_prefix")
load("@rules_pkg//pkg:tar.bzl", "pkg_tar")
load("//bazel/portage/build_defs:common.bzl", "ExtraSourcesInfo")

visibility("public")

def _extra_sources_provider_impl(ctx):
    return [
        DefaultInfo(files = depset([ctx.file.tar])),
        ExtraSourcesInfo(tar = ctx.file.tar),
    ]

_extra_sources_provider = rule(
    implementation = _extra_sources_provider_impl,
    attrs = {
        "tar": attr.label(allow_single_file = True),
    },
    provides = [ExtraSourcesInfo],
    doc = "Returns ExtraSourcesInfo.",
)

def extra_sources(*, name, srcs):
    """Defines extra sources that can be provided on building packages.

    Use this macro to define a set of files to be provided as extra sources on
    building certain packages. You can refer to the target in TOML files
    associated with ebuilds or eclasses. Then those files will be accessible
    under /mnt/host/source/src on building those packages.

    This macro doesn't accept the "visibility" argument. Instead, an appropriate
    visibility is set so that the target can be referred to by generated
    repositories.

    Internally, this macro is a thin wrapper of rules_pkg, but it provides a
    safe subset of its functionalities so that users are less likely to misuse
    them. Particularly, the directory prefix is automatically derived from the
    current package path and there is no way to override it, it's forbidden to
    rename files on creating tarballs, and this macro is the only way to define
    valid targets for extra sources as it generates a private provider. This
    restriction ensures that the source code layout in ephemeral containers does
    not deviate from that of Portage-based builds.

    Args:
        name: The name of the target.
        srcs: Source files to be provided.
    """
    pkg_files(
        name = "%s_files" % name,
        srcs = srcs,
        attributes = pkg_attributes(mode = "0755"),
        prefix = "/mnt/host/source/src",
        strip_prefix = strip_prefix.from_root(),
        visibility = ["//visibility:private"],
    )

    pkg_tar(
        name = "%s_tar" % name,
        srcs = [":%s_files" % name],
        compressor = "@//bazel/portage/repo_defs/zstd",
        compressor_args = "--threads=0",
        extension = "tar.zst",
        visibility = ["//visibility:private"],
    )

    _extra_sources_provider(
        name = name,
        tar = ":%s_tar" % name,
        visibility = ["@//bazel:internal"],
    )
