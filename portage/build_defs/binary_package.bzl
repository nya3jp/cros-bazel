# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "BinaryPackageInfo", "single_binary_package_set_info")
load("package_contents.bzl", "generate_contents")

def _binary_package_impl(ctx):
    src = ctx.file.src
    src_basename = src.basename.rsplit(".", 1)[0]

    contents = generate_contents(
        ctx = ctx,
        binary_package = src,
        output_prefix = src_basename,
        # Currently all usage of the binary_package rule is for host packages.
        board = None,
        executable_action_wrapper = ctx.executable._action_wrapper,
        executable_extract_package = ctx.executable._extract_package,
    )

    direct_runtime_deps = tuple([
        target[BinaryPackageInfo]
        for target in ctx.attr.runtime_deps
    ])
    transitive_runtime_deps = depset(
        transitive = [
            depset(
                [dep],
                transitive = [dep.transitive_runtime_deps],
                order = "postorder",
            )
            for dep in direct_runtime_deps
        ],
        order = "postorder",
    )
    all_files = depset(
        [src],
        transitive = [pkg.all_files for pkg in direct_runtime_deps],
        order = "postorder",
    )
    package_info = BinaryPackageInfo(
        file = src,
        contents = contents,
        all_files = all_files,
        package_name = ctx.attr.package_name or ctx.label.name,
        category = ctx.attr.category,
        version = ctx.attr.version,
        direct_runtime_deps = direct_runtime_deps,
        transitive_runtime_deps = transitive_runtime_deps,
        layer = None,
    )
    package_set_info = single_binary_package_set_info(package_info)
    return [
        DefaultInfo(files = depset([src])),
        package_info,
        package_set_info,
    ]

binary_package = rule(
    implementation = _binary_package_impl,
    attrs = {
        "category": attr.string(mandatory = True),
        "package_name": attr.string(mandatory = True),
        "runtime_deps": attr.label_list(
            providers = [BinaryPackageInfo],
        ),
        "src": attr.label(
            mandatory = True,
            allow_single_file = [".tbz2"],
        ),
        "version": attr.string(mandatory = True),
        "_action_wrapper": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/action_wrapper"),
        ),
        "_extract_package": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/extract_package"),
        ),
    },
)
