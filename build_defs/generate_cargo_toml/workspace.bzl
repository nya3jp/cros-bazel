# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Generation of cargo workspaces from bazel workspaces."""

load("//bazel/build_defs/jinja_template:render_template.bzl", "render_template")
load(":workspace_metadata.bzl", "WorkspaceInfo")

visibility("//bazel")

def _generate_workspace_vars_impl(ctx):
    out = ctx.actions.declare_file(ctx.label.name + ".json")
    args = ctx.actions.args()
    args.add("--out", out)
    workspace = ctx.attr.workspace[WorkspaceInfo]
    args.add("--manifest", workspace.manifest)
    args.add_all("--members", [member.manifest for member in workspace.members])

    ctx.actions.run(
        executable = ctx.executable._generate_workspace_vars,
        arguments = [args],
        # Don't include the member Cargo.toml files.
        # We depend on their file paths, but don't ever read the files.
        inputs = [workspace.manifest],
        outputs = [out],
    )

    return [DefaultInfo(files = depset([out]))]

_generate_workspace_vars = rule(
    implementation = _generate_workspace_vars_impl,
    attrs = dict(
        workspace = attr.label(providers = [WorkspaceInfo], mandatory = True),
        _generate_workspace_vars = attr.label(
            default = "//bazel/build_defs/generate_cargo_toml:generate_workspace_vars",
            executable = True,
            cfg = "exec",
        ),
    ),
)

def cargo_workspace_toml(*, name, **kwargs):
    vars_name = "_%s_vars" % name
    _generate_workspace_vars(
        name = vars_name,
        visibility = ["//visibility:private"],
        **kwargs
    )

    render_template(
        name = name,
        template = "//bazel/build_defs/generate_cargo_toml:workspace_cargo_toml",
        out = "Cargo_generated.toml",
        vars_file = vars_name,
        regen_name = "generate_cargo_files",
        visibility = ["//visibility:private"],
        testonly = True,
    )

def _cargo_workspace_lock_impl(ctx):
    out = ctx.actions.declare_file(ctx.label.name + ".lock")
    args = ctx.actions.args()
    args.add("--out", out)
    workspace = ctx.attr.workspace[WorkspaceInfo]
    manifests = [member.manifest for member in workspace.members]
    root_manifest = ctx.file.manifest
    args.add("--lockfile", workspace.lockfile)
    args.add("--root_manifest", root_manifest)
    args.add_all("--manifests", manifests)
    args.add_all(
        "--srcs",
        depset(transitive = [
            member.srcs
            for member in workspace.members
        ]).to_list(),
    )

    ctx.actions.run(
        executable = ctx.executable._generate_lockfile,
        arguments = [args],
        # Intentionally don't add the srcs, since we only care about the paths.
        inputs = manifests + [workspace.lockfile, root_manifest],
        outputs = [out],
    )

    return [DefaultInfo(files = depset([out]))]

cargo_workspace_lock = rule(
    implementation = _cargo_workspace_lock_impl,
    attrs = dict(
        manifest = attr.label(allow_single_file = [".toml"], mandatory = True),
        workspace = attr.label(providers = [WorkspaceInfo], mandatory = True),
        _generate_lockfile = attr.label(
            default = "//bazel/build_defs/generate_cargo_toml:generate_workspace_lock",
            executable = True,
            cfg = "exec",
        ),
    ),
)
