# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "EbuildSrcInfo", "relative_path_in_label")
load("@bazel_skylib//lib:paths.bzl", "paths")

def _format_create_squashfs_arg(file, package):
    return "%s:%s" % (relative_path_in_label(file, package), file.path)

def _create_squashfs_action(ctx, out, exe, files):
    # We want to support adding files from sub packages, so we need to use
    # the target's path instead of the file owners path when computing
    # the relative path.
    package = ctx.label

    args = ctx.actions.args()
    args.add_all(files,
        allow_closure = True,
        map_each = lambda file: _format_create_squashfs_arg(file, package))
    args.set_param_file_format("multiline")
    args.use_param_file("--specs-from=%s", use_always = True)

    ctx.actions.run(
        inputs = [exe] + files,
        outputs = [out],
        executable = exe.path,
        arguments = ["--output=" + out.path, args],
    )

def _ebuild_src_impl(ctx):
    out = ctx.actions.declare_file(ctx.attr.name + ".squashfs")

    _create_squashfs_action(ctx, out, ctx.executable._create_squashfs, ctx.files.srcs)

    if ctx.attr.mount_path:
        mount_path = ctx.attr.mount_path
    else:
        mount_path = paths.join("src", ctx.label.package)

    return [
        DefaultInfo(files = depset([out])),
        EbuildSrcInfo(file = out, mount_path = mount_path),
    ]

ebuild_src = rule(
    implementation = _ebuild_src_impl,
    attrs = {
        "srcs": attr.label_list(
            allow_files = True,
            mandatory = True,
        ),
        "mount_path": attr.string(
            doc= "Path inside the container to mount the src." +
            "This value will default to src/<package path>."
        ),
        "_create_squashfs": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/create_squashfs"),
        ),
    },
)
