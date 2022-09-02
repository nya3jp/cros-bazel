# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "BinaryPackageInfo", "OverlaySetInfo", "SDKInfo", "relative_path_in_package")

def _format_file_arg(file):
    return "--file=%s=%s" % (relative_path_in_package(file), file.path)

def _ebuild_impl(ctx):
    src_basename = ctx.file.src.basename.rsplit(".", 1)[0]
    output = ctx.actions.declare_file(src_basename + ".tbz2")
    sdk = ctx.attr._sdk[SDKInfo]

    args = ctx.actions.args()
    args.add_all([
        "--ebuild=" + ctx.file.src.path,
        "--category=" + ctx.attr.category,
        "--output=" + output.path,
        "--board=" + sdk.board,
    ])

    direct_inputs = [
        ctx.executable._build_package,
        ctx.file.src,
    ]
    transitive_inputs = []

    args.add_all(sdk.squashfs_files, format_each = "--sdk=%s")
    direct_inputs.extend(sdk.squashfs_files)

    for file in ctx.attr.files:
        args.add_all(file.files, map_each = _format_file_arg)
        transitive_inputs.append(file.files)

    for distfile, name in ctx.attr.distfiles.items():
        files = distfile.files.to_list()
        if len(files) != 1:
            fail("cannot refer to multi-file rule in distfiles")
        file = files[0]
        args.add("--distfile=%s=%s" % (name, file.path))
        direct_inputs.append(file)

    overlays = ctx.attr._overlays[OverlaySetInfo].overlays
    for overlay in overlays:
        args.add("--overlay=%s=%s" % (overlay.mount_path, overlay.squashfs_file.path))
        direct_inputs.append(overlay.squashfs_file)

    # TODO: Consider target/host transitions.
    build_deps = depset(
        [dep[BinaryPackageInfo].file for dep in ctx.attr.build_target_deps],
        order = "postorder",
    )
    runtime_deps = depset(
        [output],
        transitive = [dep[BinaryPackageInfo].runtime_deps for dep in ctx.attr.runtime_deps],
        order = "postorder",
    )

    args.add_all(build_deps, format_each = "--install-target=%s")
    transitive_inputs.append(build_deps)

    ctx.actions.run(
        inputs = depset(direct_inputs, transitive = transitive_inputs),
        outputs = [output],
        executable = ctx.executable._build_package,
        arguments = [args],
        mnemonic = "Ebuild",
        progress_message = "Building " + ctx.file.src.basename,
    )

    return [
        DefaultInfo(files = depset([output])),
        BinaryPackageInfo(
            file = output,
            runtime_deps = runtime_deps,
        ),
    ]

ebuild = rule(
    implementation = _ebuild_impl,
    attrs = {
        "src": attr.label(
            mandatory = True,
            allow_single_file = [".ebuild"],
        ),
        "category": attr.string(
            mandatory = True,
        ),
        "distfiles": attr.label_keyed_string_dict(
            allow_files = True,
        ),
        "build_target_deps": attr.label_list(
            providers = [BinaryPackageInfo],
        ),
        "runtime_deps": attr.label_list(
            providers = [BinaryPackageInfo],
        ),
        "files": attr.label_list(
            allow_files = True,
        ),
        "_overlays": attr.label(
            providers = [OverlaySetInfo],
            default = "//bazel/config:overlays",
        ),
        "_build_package": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/build_package"),
        ),
        "_sdk": attr.label(
            providers = [SDKInfo],
            default = Label("//bazel/sdk"),
        ),
    },
)
