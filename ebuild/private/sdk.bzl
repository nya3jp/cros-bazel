# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "BinaryPackageInfo", "OverlaySetInfo", "SDKInfo")

def _sdk_from_archive_impl(ctx):
    output_root = ctx.actions.declare_directory(ctx.attr.name)
    output_symindex = ctx.actions.declare_file(ctx.attr.name + ".symindex")

    args = ctx.actions.args()
    args.add_all([
        "--input=" + ctx.file.src.path,
        "--output-dir=" + output_root.path,
        "--output-symindex=" + output_symindex.path,
    ])

    inputs = [ctx.executable._sdk_from_archive, ctx.file.src]
    outputs = [output_root, output_symindex]

    ctx.actions.run(
        inputs = inputs,
        outputs = outputs,
        executable = ctx.executable._sdk_from_archive,
        arguments = [args],
        mnemonic = "SdkFromArchive",
        progress_message = "Extracting SDK archive",
    )

    return [
        DefaultInfo(files = depset(outputs)),
        SDKInfo(
            board = ctx.attr.board,
            layers = [output_root, output_symindex],
        )
    ]

sdk_from_archive = rule(
    implementation = _sdk_from_archive_impl,
    attrs = {
        "src": attr.label(
            mandatory = True,
            allow_single_file = True,
        ),
        "board": attr.string(
            mandatory = True,
        ),
        "_sdk_from_archive": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/sdk_from_archive"),
        ),
    },
)

def _sdk_update_impl(ctx):
    base_sdk = ctx.attr.base[SDKInfo]

    output_root = ctx.actions.declare_directory(ctx.attr.name)
    output_symindex = ctx.actions.declare_file(ctx.attr.name + ".symindex")

    host_installs = depset(
        transitive = [label[BinaryPackageInfo].transitive_runtime_deps_files for label in ctx.attr.host_deps],
    )
    target_installs = depset(
        transitive = [label[BinaryPackageInfo].transitive_runtime_deps_files for label in ctx.attr.target_deps],
    )

    args = ctx.actions.args()
    args.add_all([
        "--board=" + base_sdk.board,
        "--output-dir=" + output_root.path,
        "--output-symindex=" + output_symindex.path,
    ])
    args.add_all(base_sdk.layers, format_each = "--input=%s", expand_directories = False)
    args.add_all(host_installs, format_each = "--install-host=%s")
    args.add_all(target_installs, format_each = "--install-target=%s")
    args.add_all(ctx.files.extra_tarballs, format_each = "--install-tarball=%s")

    overlay_inputs = []
    for overlay in ctx.attr._overlays[OverlaySetInfo].overlays:
        args.add("--overlay=%s=%s" % (overlay.mount_path, overlay.squashfs_file.path))
        overlay_inputs.append(overlay.squashfs_file)

    inputs = depset(
        [ctx.executable._sdk_update] + base_sdk.layers + overlay_inputs + ctx.files.extra_tarballs,
        transitive = [host_installs, target_installs],
    )

    outputs = [output_root, output_symindex]

    ctx.actions.run(
        inputs = inputs,
        outputs = outputs,
        executable = ctx.executable._sdk_update,
        arguments = [args],
        mnemonic = "SdkUpdate",
        progress_message = "Updating SDK",
    )

    return [
        DefaultInfo(files = depset(outputs)),
        SDKInfo(
            board = base_sdk.board,
            layers = [output_root, output_symindex] + base_sdk.layers,
        ),
    ]

sdk_update = rule(
    implementation = _sdk_update_impl,
    attrs = {
        "base": attr.label(
            mandatory = True,
            providers = [SDKInfo],
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
        "_sdk_update": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/sdk_update"),
        ),
        "_overlays": attr.label(
            providers = [OverlaySetInfo],
            default = "//bazel/config:overlays",
        ),
    },
)
