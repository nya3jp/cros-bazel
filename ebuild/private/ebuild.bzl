# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "BinaryPackageInfo", "EbuildSrcInfo", "OverlaySetInfo", "SDKInfo", "relative_path_in_package")
load("@bazel_skylib//lib:paths.bzl", "paths")

def _format_file_arg(file):
    return "--file=%s=%s" % (relative_path_in_package(file), file.path)

def _ebuild_impl(ctx):
    src_basename = ctx.file.ebuild.basename.rsplit(".", 1)[0]
    output = ctx.actions.declare_file(src_basename + ".tbz2")
    sdk = ctx.attr._sdk[SDKInfo]

    args = ctx.actions.args()
    args.add_all([
        "--ebuild=" + ctx.file.ebuild.path,
        "--category=" + ctx.attr.category,
        "--output=" + output.path,
        "--board=" + sdk.board,
    ])

    direct_inputs = [
        ctx.executable._build_package,
        ctx.file.ebuild,
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

    for target in ctx.attr.srcs:
        info = target[EbuildSrcInfo]
        args.add("--overlay=src/%s=%s" % (info.src_path, info.squashfs_file.path))
        direct_inputs.append(info.squashfs_file)

    # TODO: Consider target/host transitions.
    build_deps = depset(
        # Pull in runtime dependencies of build-time dependencies.
        # TODO: Revisit this logic to see if we can avoid pulling in transitive
        # dependencies.
        transitive = [dep[BinaryPackageInfo].runtime_deps for dep in ctx.attr.build_deps],
        order = "postorder",
    )
    runtime_deps = depset(
        [output],
        transitive = [dep[BinaryPackageInfo].runtime_deps for dep in ctx.attr.runtime_deps],
        order = "postorder",
    )

    args.add_all(build_deps, format_each = "--install-target=%s")
    transitive_inputs.append(build_deps)

    package_name = "%s/%s" % (ctx.attr.category, paths.split_extension(ctx.file.ebuild.basename)[0])

    ctx.actions.run(
        inputs = depset(direct_inputs, transitive = transitive_inputs),
        outputs = [output],
        executable = ctx.executable._build_package,
        arguments = [args],
        mnemonic = "Ebuild",
        progress_message = "Building " + package_name,
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
        "ebuild": attr.label(
            mandatory = True,
            allow_single_file = [".ebuild"],
        ),
        "category": attr.string(
            mandatory = True,
        ),
        "distfiles": attr.label_keyed_string_dict(
            allow_files = True,
        ),
        "srcs": attr.label_list(
            doc="src files used by the ebuild",
            providers = [EbuildSrcInfo]
        ),
        "build_deps": attr.label_list(
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
