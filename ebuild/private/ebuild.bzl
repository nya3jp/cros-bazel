# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "BinaryPackageInfo", "EbuildSrcInfo", "OverlaySetInfo", "SDKInfo", "relative_path_in_package")
load("@bazel_skylib//lib:paths.bzl", "paths")

def _format_file_arg(file):
    return "--file=%s=%s" % (relative_path_in_package(file), file.path)

def _map_install_group(targets):
    files = []
    for target in targets:
        file = target[BinaryPackageInfo].file
        files.append(file.path)
    return ":".join(files)

def _calculate_install_groups(build_deps):
    seen = {}

    # An ordered list containing a list of deps that can be installed in parallel
    levels = []

    remaining_targets = build_deps.to_list()

    for _ in range(100):
        if len(remaining_targets) == 0:
            break

        satisfied_list = []
        not_satisfied_list = []
        for target in remaining_targets:
            info = target[BinaryPackageInfo]

            all_seen = True
            for runtime_target in info.direct_runtime_deps_targets:
                if not seen.get(runtime_target.label):
                    all_seen = False
                    break

            if all_seen:
                satisfied_list.append(target)
            else:
                not_satisfied_list.append(target)

        if len(satisfied_list) == 0:
            fail("Dependency list is unsatisfiable")

        for target in satisfied_list:
            seen[target.label] = True

        levels.append(satisfied_list)
        remaining_targets = not_satisfied_list

    if len(remaining_targets) > 0:
        fail("Too many dependencies")

    return levels

def _ebuild_impl(ctx):
    src_basename = ctx.file.ebuild.basename.rsplit(".", 1)[0]
    output = ctx.actions.declare_file(src_basename + ".tbz2")
    sdk = ctx.attr._sdk[SDKInfo]

    args = ctx.actions.args()
    args.add_all([
        "--ebuild=" + ctx.file.ebuild.path,
        "--output=" + output.path,
        "--board=" + sdk.board,
    ])

    direct_inputs = [
        ctx.executable._build_package,
        ctx.file.ebuild,
    ]
    transitive_inputs = []

    args.add_all(sdk.layers, format_each = "--sdk=%s", expand_directories = False)
    direct_inputs.extend(sdk.layers)

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
        if target.label.workspace_name == "chromite" and target.label.package == "" and target.label.name == "src":
            args.add("--overlay=chromite=%s" % (info.squashfs_file.path))
        else:
            args.add("--overlay=src/%s=%s" % (info.src_path, info.squashfs_file.path))
        direct_inputs.append(info.squashfs_file)

    # TODO: Consider target/host transitions.
    transitive_build_time_deps_files = depset(
        # Pull in runtime dependencies of build-time dependencies.
        # TODO: Revisit this logic to see if we can avoid pulling in transitive
        # dependencies.
        transitive = [dep[BinaryPackageInfo].transitive_runtime_deps_files for dep in ctx.attr.build_deps],
        order = "postorder",
    )

    transitive_inputs.append(transitive_build_time_deps_files)

    transitive_build_time_deps_targets = depset(
        ctx.attr.build_deps,
        # Pull in runtime dependencies of build-time dependencies.
        # TODO: Revisit this logic to see if we can avoid pulling in transitive
        # dependencies.
        transitive = [dep[BinaryPackageInfo].transitive_runtime_deps_targets for dep in ctx.attr.build_deps],
        order = "postorder",
    )

    install_groups = _calculate_install_groups(transitive_build_time_deps_targets)
    args.add_all(install_groups, map_each = _map_install_group, format_each = "--install-target=%s")

    transitive_runtime_deps_files = depset(
        [output],
        transitive = [dep[BinaryPackageInfo].transitive_runtime_deps_files for dep in ctx.attr.runtime_deps],
        order = "postorder",
    )

    transitive_runtime_deps_targets = depset(
        ctx.attr.runtime_deps,
        transitive = [dep[BinaryPackageInfo].transitive_runtime_deps_targets for dep in ctx.attr.runtime_deps],
        order = "postorder",
    )

    ctx.actions.run(
        inputs = depset(direct_inputs, transitive = transitive_inputs),
        outputs = [output],
        executable = ctx.executable._build_package,
        arguments = [args],
        mnemonic = "Ebuild",
        progress_message = "Building " + ctx.file.ebuild.basename,
    )

    return [
        DefaultInfo(files = depset([output])),
        BinaryPackageInfo(
            file = output,
            transitive_runtime_deps_files = transitive_runtime_deps_files,
            transitive_runtime_deps_targets = transitive_runtime_deps_targets,
            direct_runtime_deps_targets = ctx.attr.runtime_deps,
        ),
    ]

_ebuild = rule(
    implementation = _ebuild_impl,
    attrs = {
        "ebuild": attr.label(
            mandatory = True,
            allow_single_file = [".ebuild"],
        ),
        "distfiles": attr.label_keyed_string_dict(
            allow_files = True,
        ),
        "srcs": attr.label_list(
            doc = "src files used by the ebuild",
            providers = [EbuildSrcInfo],
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

def ebuild(*, tags=[], **kwargs):
    # Disable sandbox to avoid creating a symlink forest.
    # This does not affect hermeticity since ebuild runs in a container.
    tags = ["no-sandbox"] + tags
    _ebuild(tags=tags, **kwargs)
