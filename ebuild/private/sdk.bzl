# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "BinaryPackageSetInfo", "OverlaySetInfo", "SDKInfo")
load("install_deps.bzl", "install_deps")

def _sdk_from_archive_impl(ctx):
    output_root = ctx.actions.declare_directory(ctx.attr.name)
    output_log = ctx.actions.declare_file(ctx.attr.name + ".log")

    args = ctx.actions.args()
    args.add_all([
        "--output=" + output_log.path,
        ctx.executable._sdk_from_archive,
        "--input=" + ctx.file.src.path,
        "--output=" + output_root.path,
    ])

    inputs = [ctx.executable._sdk_from_archive, ctx.file.src]
    outputs = [output_root, output_log]

    ctx.actions.run(
        inputs = inputs,
        outputs = outputs,
        executable = ctx.executable._action_wrapper,
        tools = [ctx.executable._sdk_from_archive],
        arguments = [args],
        mnemonic = "SdkFromArchive",
        progress_message = "Extracting SDK archive",
    )

    return [
        DefaultInfo(files = depset(outputs)),
        SDKInfo(
            layers = [output_root],
        ),
    ]

sdk_from_archive = rule(
    implementation = _sdk_from_archive_impl,
    attrs = {
        "src": attr.label(
            mandatory = True,
            allow_single_file = True,
        ),
        "_action_wrapper": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/action_wrapper"),
        ),
        "_sdk_from_archive": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/sdk_from_archive"),
        ),
    },
)

def _sdk_update_impl(ctx):
    output_root = ctx.actions.declare_directory(ctx.attr.name)
    output_log = ctx.actions.declare_file(ctx.attr.name + ".log")

    host_installs = depset(
        transitive = [
            target[BinaryPackageSetInfo].files
            for target in ctx.attr.host_deps
        ],
    )
    target_installs = depset(
        transitive = [
            target[BinaryPackageSetInfo].files
            for target in ctx.attr.target_deps
        ],
    )

    args = ctx.actions.args()
    args.add_all([
        "--output=" + output_log.path,
        ctx.executable._sdk_update,
        "--board=" + ctx.attr.board,
        "--output=" + output_root.path,
    ])

    base_sdk = ctx.attr.base[SDKInfo]
    overlays = ctx.attr.overlays[OverlaySetInfo]
    layer_inputs = base_sdk.layers + overlays.layers
    args.add_all(layer_inputs, format_each = "--layer=%s", expand_directories = False)

    args.add_all(host_installs, format_each = "--install-host=%s")
    args.add_all(target_installs, format_each = "--install-target=%s")
    args.add_all(ctx.files.extra_tarballs, format_each = "--install-tarball=%s")

    inputs = depset(
        [ctx.executable._sdk_update] + layer_inputs + ctx.files.extra_tarballs,
        transitive = [host_installs, target_installs],
    )

    outputs = [output_root, output_log]

    ctx.actions.run(
        inputs = inputs,
        outputs = outputs,
        executable = ctx.executable._action_wrapper,
        tools = [ctx.executable._sdk_update],
        arguments = [args],
        execution_requirements = {
            # Send SIGTERM instead of SIGKILL on user interruption.
            "supports-graceful-termination": "",
            # Disable sandbox to avoid creating a symlink forest.
            # This does not affect hermeticity since sdk_update runs in a container.
            "no-sandbox": "",
        },
        mnemonic = "SdkUpdate",
        progress_message = "Updating SDK",
    )

    return [
        DefaultInfo(files = depset(outputs)),
        SDKInfo(
            layers = base_sdk.layers + [output_root],
        ),
    ]

sdk_update = rule(
    implementation = _sdk_update_impl,
    attrs = {
        "base": attr.label(
            mandatory = True,
            providers = [SDKInfo],
        ),
        "board": attr.string(
            mandatory = True,
            doc = """
            The target board name to build the package for.
            """,
        ),
        "host_deps": attr.label_list(
            providers = [BinaryPackageSetInfo],
        ),
        "target_deps": attr.label_list(
            providers = [BinaryPackageSetInfo],
        ),
        "extra_tarballs": attr.label_list(
            allow_files = True,
        ),
        "overlays": attr.label(
            providers = [OverlaySetInfo],
            mandatory = True,
        ),
        "_action_wrapper": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/action_wrapper"),
        ),
        "_sdk_update": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/sdk_update"),
        ),
    },
)

def _sdk_install_deps_impl(ctx):
    sdk = ctx.attr.base[SDKInfo]

    install_set = depset(
        transitive = [dep[BinaryPackageSetInfo].packages for dep in ctx.attr.target_deps],
        order = "postorder",
    )

    if "{dep_count}" in ctx.attr.progress_message:
        progress_message = ctx.attr.progress_message.replace(
            "{dep_count}",
            str(len(install_set.to_list())),
        )
    else:
        progress_message = ctx.attr.progress_message

    outputs = install_deps(
        ctx = ctx,
        output_prefix = ctx.attr.name,
        board = ctx.attr.board,
        sdk = sdk,
        overlays = ctx.attr.overlays[OverlaySetInfo],
        install_set = install_set,
        executable_action_wrapper = ctx.executable._action_wrapper,
        executable_install_deps = ctx.executable._install_deps,
        progress_message = progress_message,
    )

    return [
        DefaultInfo(files = depset(outputs)),
        SDKInfo(
            layers = sdk.layers + outputs,
        ),
    ]

sdk_install_deps = rule(
    implementation = _sdk_install_deps_impl,
    attrs = {
        "base": attr.label(
            doc = """
            Base SDK to derive a new SDK from.
            """,
            mandatory = True,
            providers = [SDKInfo],
        ),
        "board": attr.string(
            mandatory = True,
            doc = """
            The target board name to build the package for.
            """,
        ),
        "overlays": attr.label(
            providers = [OverlaySetInfo],
            mandatory = True,
        ),
        "target_deps": attr.label_list(
            doc = """
            Target packages to install in the SDK.
            """,
            providers = [BinaryPackageSetInfo],
        ),
        "progress_message": attr.string(
            doc = """
            Progress message for this target.
            If the message contains `{dep_count}' it will be replaced with the
            total number of dependencies that need to be installed.
            """,
            default = "Updating SDK",
        ),
        "_action_wrapper": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/action_wrapper"),
        ),
        "_install_deps": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/install_deps"),
        ),
    },
)
