# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@rules_pkg//pkg:providers.bzl", "PackageFilegroupInfo")
load("//bazel/bash:defs.bzl", "wrap_binary_with_args", _runfiles_path = "runfiles_path")

_PREFIX = """
test -z "${RUNFILES_DIR:-}" && export RUNFILES_MANIFEST_ONLY=1

if [[ $(id -u) -ne 0 ]]; then
  exec sudo "${BASH_SOURCE[0]}" "$@"
fi
"""

def runfiles_path(ctx, f):
    path = _runfiles_path(ctx, f)
    if path.startswith("_main/"):
        return "cros/" + path[6:]
    return path

def _deploy_local_impl(ctx):
    metadata = ctx.attr.filegroup[PackageFilegroupInfo]
    dirs = [x[0] for x in metadata.pkg_dirs]
    symlinks = [x[0] for x in metadata.pkg_symlinks]
    files = []
    for file, _ in metadata.pkg_files:
        files.append(dict(
            attributes = file.attributes,
            dest_src_map = {
                k: runfiles_path(ctx, v)
                for k, v in file.dest_src_map.items()
            },
        ))
    manifest_content = json.encode(dict(
        dirs = dirs,
        symlinks = symlinks,
        files = files,
    ))
    manifest = ctx.actions.declare_file(ctx.label.name + "_manifest.json")
    ctx.actions.write(output = manifest, content = manifest_content)

    binary = ctx.actions.declare_file(ctx.label.name)

    return wrap_binary_with_args(
        ctx,
        out = binary,
        binary = ctx.attr._deploy,
        args = ["--manifest=" + runfiles_path(ctx, manifest)],
        content_prefix = _PREFIX,
        runfiles = ctx.runfiles(
            files = [manifest],
            transitive_files = ctx.attr.filegroup[DefaultInfo].files,
        ),
    )

deploy_local = rule(
    implementation = _deploy_local_impl,
    attrs = dict(
        filegroup = attr.label(mandatory = True),
        _deploy = attr.label(executable = True, default = "//bazel/cros_pkg/private:deploy_local", cfg = "exec"),
        _bash_runfiles = attr.label(default = "@bazel_tools//tools/bash/runfiles"),
    ),
    executable = True,
)
