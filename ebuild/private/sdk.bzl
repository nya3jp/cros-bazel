# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "BinaryPackageInfo", "OverlaySetInfo", "SDKInfo")

def _sdk_impl(ctx):
    base_squashfs_output = ctx.actions.declare_file(ctx.attr.name + "_base.squashfs")
    pkgs_squashfs_output = ctx.actions.declare_file(ctx.attr.name + "_pkgs.squashfs")
    host_installs = [label[BinaryPackageInfo].file for label in ctx.attr.host_deps]
    target_installs = [label[BinaryPackageInfo].file for label in ctx.attr.target_deps]

    ctx.actions.run_shell(
        outputs = [base_squashfs_output],
        inputs = [ctx.file.src],
        # TODO: Don't depend on system binaries (xzcat, mksquashfs).
        # TODO: Avoid -all-root.
        command = "xzcat \"$1\" | mksquashfs - \"$2\" -tar -all-time 0 -all-root",
        arguments = [ctx.file.src.path, base_squashfs_output.path],
        progress_message = "Converting %{input} to squashfs",
    )

    args = ctx.actions.args()
    args.add_all([
        "--input-squashfs=" + base_squashfs_output.path,
        "--output-squashfs=" + pkgs_squashfs_output.path,
        "--board=" + ctx.attr.board,
    ])
    args.add_all(host_installs, format_each = "--install-host=%s")
    args.add_all(target_installs, format_each = "--install-target=%s")

    overlay_inputs = []
    for overlay in ctx.attr._overlays[OverlaySetInfo].overlays:
        args.add("--overlay=%s=%s" % (overlay.mount_path, overlay.squashfs_file.path))
        overlay_inputs.append(overlay.squashfs_file)

    inputs = [
        ctx.executable._build_sdk,
        base_squashfs_output,
    ] + host_installs + target_installs + overlay_inputs

    ctx.actions.run(
        inputs = inputs,
        outputs = [pkgs_squashfs_output],
        executable = ctx.executable._build_sdk,
        arguments = [args],
        mnemonic = "Sdk",
        progress_message = "Building SDK",
    )

    outputs = [pkgs_squashfs_output, base_squashfs_output]
    return [
        DefaultInfo(files = depset(outputs)),
        SDKInfo(board = ctx.attr.board, squashfs_files = outputs),
    ]

sdk = rule(
    implementation = _sdk_impl,
    attrs = {
        "src": attr.label(
            mandatory = True,
            allow_single_file = True,
        ),
        "board": attr.string(
            mandatory = True,
        ),
        "host_deps": attr.label_list(
            providers = [BinaryPackageInfo],
        ),
        "target_deps": attr.label_list(
            providers = [BinaryPackageInfo],
        ),
        "_build_sdk": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/build_sdk"),
        ),
        "_overlays": attr.label(
            providers = [OverlaySetInfo],
            default = "//bazel/config:overlays",
        ),
    },
)
