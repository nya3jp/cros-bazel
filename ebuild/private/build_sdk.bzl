# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "BinaryPackageSetInfo", "OverlaySetInfo", "SDKInfo")
load("install_groups.bzl", "calculate_install_groups", "map_install_group")

def _build_sdk_impl(ctx):
    sdk = ctx.attr.sdk[SDKInfo]

    install_set = depset(
        transitive = [dep[BinaryPackageSetInfo].packages for dep in ctx.attr.target_deps],
        order = "postorder",
    )

    install_list = install_set.to_list()

    progress_message = ctx.attr.progress_message.replace(
        "{dep_count}",
        str(len(install_list)),
    )

    output_prefix = ctx.attr.name

    output_sdk = ctx.actions.declare_file(output_prefix + ".tar.zst")
    output_log_file = ctx.actions.declare_file(output_prefix + ".log")

    args = ctx.actions.args()
    args.add_all([
        "--output=" + output_log_file.path,
        ctx.executable._build_sdk.path,
        "--board=" + ctx.attr.board,
        "--output=" + output_sdk.path,
    ])

    direct_inputs = [pkg.file for pkg in install_list]

    layer_inputs = sdk.layers + ctx.attr.overlays[OverlaySetInfo].layers + ctx.files.extra_tarballs
    args.add_all(layer_inputs, format_each = "--layer=%s", expand_directories = False)
    direct_inputs.extend(layer_inputs)

    install_groups = calculate_install_groups(install_list)
    args.add_all(install_groups, map_each = map_install_group, format_each = "--install-target=%s")

    ctx.actions.run(
        inputs = depset(direct_inputs),
        outputs = [output_sdk, output_log_file],
        executable = ctx.executable._action_wrapper,
        tools = [ctx.executable._build_sdk],
        arguments = [args],
        execution_requirements = {
            # Send SIGTERM instead of SIGKILL on user interruption.
            "supports-graceful-termination": "",
            # Disable sandbox to avoid creating a symlink forest.
            # This does not affect hermeticity since ebuild runs in a container.
            "no-sandbox": "",
        },
        mnemonic = "InstallDeps",
        progress_message = progress_message,
    )

    return [
        DefaultInfo(files = depset([output_sdk])),
        SDKInfo(
            layers = [output_sdk],
        ),
    ]

build_sdk = rule(
    implementation = _build_sdk_impl,
    attrs = {
        "sdk": attr.label(
            doc = """
            The SDK that was used to create the packages listed in target_deps.
            """,
            mandatory = True,
            providers = [SDKInfo],
        ),
        "board": attr.string(
            mandatory = True,
            doc = """
            The board name of the target SDK board.
            """,
        ),
        "overlays": attr.label(
            providers = [OverlaySetInfo],
            mandatory = True,
        ),
        "target_deps": attr.label_list(
            doc = """
            Packages that will be used to create the new SDK.
            """,
            providers = [BinaryPackageSetInfo],
        ),
        "extra_tarballs": attr.label_list(
            allow_files = True,
        ),
        "progress_message": attr.string(
            doc = """
            Progress message for this target.
            If the message contains `{dep_count}' it will be replaced with the
            total number of dependencies that need to be installed.
            """,
            default = "Building %{label} with {dep_count} packages",
        ),
        "_action_wrapper": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/action_wrapper"),
        ),
        "_build_sdk": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/build_sdk"),
        ),
    },
)