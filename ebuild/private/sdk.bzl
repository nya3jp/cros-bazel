# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "BinaryPackageInfo", "OverlaySetInfo", "SDKInfo")

def _sdk_impl(ctx):
    pkgs_dir_output = ctx.actions.declare_directory(ctx.attr.name + "_pkgs")
    pkgs_squashfs_output = ctx.actions.declare_file(ctx.attr.name + "_pkgs.squashfs")
    host_installs = depset(
        transitive = [label[BinaryPackageInfo].transitive_install_files for label in ctx.attr.host_deps],
    )
    target_installs = depset(
        transitive = [label[BinaryPackageInfo].transitive_install_files for label in ctx.attr.target_deps],
    )

    args = ctx.actions.args()
    args.add_all([
        "--input=" + ctx.file.src.path,
        "--output-dir=" + pkgs_dir_output.path,
        "--output-squashfs=" + pkgs_squashfs_output.path,
        "--board=" + ctx.attr.board,
    ])
    args.add_all(host_installs, format_each = "--install-host=%s")
    args.add_all(target_installs, format_each = "--install-target=%s")
    args.add_all(ctx.files.extra_tarballs, format_each = "--install-tarball=%s")

    overlay_inputs = []
    for overlay in ctx.attr._overlays[OverlaySetInfo].overlays:
        args.add("--overlay=%s=%s" % (overlay.mount_path, overlay.squashfs_file.path))
        overlay_inputs.append(overlay.squashfs_file)

    inputs = depset(
        [ctx.executable._build_sdk, ctx.file.src] + overlay_inputs + ctx.files.extra_tarballs,
        transitive = [host_installs, target_installs],
    )

    outputs = [pkgs_squashfs_output, pkgs_dir_output]

    ctx.actions.run(
        inputs = inputs,
        outputs = outputs,
        executable = ctx.executable._build_sdk,
        arguments = [args],
        mnemonic = "Sdk",
        progress_message = "Building SDK",
    )

    return [
        DefaultInfo(files = depset(outputs)),
        SDKInfo(board = ctx.attr.board, files = [
            pkgs_dir_output,
            pkgs_squashfs_output,
            ctx.file.src,
        ]),
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
        "extra_tarballs": attr.label_list(
            allow_files = True,
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
