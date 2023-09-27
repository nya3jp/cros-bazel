# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/bash:defs.bzl", "BASH_RUNFILES_ATTRS", "bash_rlocation", "generate_bash_script")
load("//bazel/portage/build_defs:common.bzl", "BinaryPackageSetInfo")
load(":common.bzl", "EXTRACT_COMMON_ATTRS")

visibility("//bazel/portage/build_defs")

_CONTENT = """
BUILDOZER="$(rlocation files/buildozer)"
BIN="$(rlocation cros/bazel/portage/bin/extract_package_from_manifest/update_manifest/update_manifest)"

"${{BIN}}" {args} "$@"

cd "${{BUILD_WORKSPACE_DIRECTORY}}"
"${{BUILDOZER}}" 'new_load {bzl_file} {var_name}' '{pkg}:__pkg__' || true
"${{BUILDOZER}}" 'set manifest_content {var_name}' '{pkg}:{name}' || true
cros format {manifest}
"""

def _update_manifest_impl(ctx):
    # Strip the _update suffix.
    name = ctx.label.name.rsplit("_", 1)[0]
    var_name = name.upper() + "_MANIFEST_CONTENT"
    pkg = "//" + ctx.label.package
    if ctx.label.workspace_name:
        pkg = "@" + ctx.label.workspace_name + pkg

    args = [
        "--regenerate-command",
        ctx.attr.manifest_regenerate_command,
        "--manifest-out",
        bash_rlocation(ctx, ctx.file.manifest_file),
        "--manifest-variable",
        var_name,
    ]

    binpkgs = ctx.attr.pkg[BinaryPackageSetInfo].files.to_list()
    for binpkg in binpkgs:
        args.extend(["--binpkg", bash_rlocation(ctx, binpkg)])

    for regex in ctx.attr.ld_library_path_regexes:
        args.extend(["--ld-library-path-regex", regex.replace("\\", "\\\\")])
    for regex in ctx.attr.header_file_dir_regexes:
        args.extend(["--header-file-dir-regex", regex.replace("\\", "\\\\")])

    return generate_bash_script(
        ctx,
        out = ctx.actions.declare_file(ctx.label.name),
        content = _CONTENT.format(
            args = " ".join(['"%s"' % arg for arg in args]),
            bzl_file = ctx.attr.manifest_file.label,
            var_name = var_name,
            name = name,
            pkg = pkg,
            manifest = bash_rlocation(ctx, ctx.file.manifest_file),
        ),
        runfiles = ctx.runfiles(
            files = binpkgs + [ctx.file.manifest_file],
            transitive_files = ctx.attr._buildozer[DefaultInfo].files,
        ),
        data = [ctx.attr._update_manifest],
    )

update_manifest = rule(
    implementation = _update_manifest_impl,
    attrs = BASH_RUNFILES_ATTRS | EXTRACT_COMMON_ATTRS | dict(
        manifest_file = attr.label(allow_single_file = [".bzl"], mandatory = True),
        _buildozer = attr.label(default = "@files//:buildozer_symlink"),
        _update_manifest = attr.label(
            executable = True,
            default = "//bazel/portage/bin/extract_package_from_manifest/update_manifest",
            cfg = "exec",
        ),
    ),
    executable = True,
)
