# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "OverlayInfo", "OverlaySetInfo", "relative_path_in_package")

def _format_create_squashfs_arg(file):
    return "%s:%s" % (relative_path_in_package(file), file.path)

def _create_squashfs_action(ctx, out, exe, files):
    args = ctx.actions.args()
    args.add_all(files, map_each = _format_create_squashfs_arg)
    args.set_param_file_format("multiline")
    args.use_param_file("--specs-from=%s", use_always = True)

    ctx.actions.run(
        inputs = [exe] + files,
        outputs = [out],
        executable = exe.path,
        arguments = ["--output=" + out.path, args],
    )

def _overlay_impl(ctx):
    out = ctx.actions.declare_file(ctx.attr.name + ".squashfs")

    _create_squashfs_action(ctx, out, ctx.executable._create_squashfs, ctx.files.srcs)

    return [
        DefaultInfo(files = depset([out])),
        OverlayInfo(squashfs_file = out, mount_path = ctx.attr.mount_path),
    ]

overlay = rule(
    implementation = _overlay_impl,
    attrs = {
        "srcs": attr.label_list(
            allow_files = True,
            mandatory = True,
        ),
        "mount_path": attr.string(
            mandatory = True,
        ),
        "_create_squashfs": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/create_squashfs"),
        ),
    },
)

def _overlay_set_impl(ctx):
    return [
        OverlaySetInfo(
            overlays = [overlay[OverlayInfo] for overlay in ctx.attr.overlays],
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
