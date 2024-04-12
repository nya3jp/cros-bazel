# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Metadata extraction for cargo workspaces."""

visibility("//bazel")

MemberInfo = provider(
    """Metadata about a cargo workspace member.""",
    fields = dict(
        manifest = "(File) Cargo.toml file",
        srcs = "(Depset[File]) The source .rs files required for the cargo.toml",
    ),
)

WorkspaceInfo = provider(
    """Metadata about a cargo workspace.""",
    fields = dict(
        members = "(List[MemberInfo]) The Cargo.toml members of the workspace",
        manifest = "(File) The root manifest file",
        lockfile = "(File) The root lockfile",
    ),
)

def _cargo_workspace_impl(ctx):
    return [WorkspaceInfo(
        members = [member[MemberInfo] for member in ctx.attr.members],
        manifest = ctx.file.manifest,
        lockfile = ctx.file.lockfile,
    )]

cargo_workspace = rule(
    doc = """Generates metadata about all bazel packages using rust.

    This is used to generate the workspace Cargo.toml / Cargo.lock files.""",
    implementation = _cargo_workspace_impl,
    attrs = dict(
        members = attr.label_list(providers = [MemberInfo], mandatory = True),
        manifest = attr.label(allow_single_file = [".toml"], mandatory = True),
        lockfile = attr.label(allow_single_file = [".lock"], mandatory = True),
    ),
    provides = [WorkspaceInfo],
)
