# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@rules_pkg//pkg:providers.bzl", "PackageArtifactInfo")
load(":common.bzl", "OverlaySetInfo", "SDKInfo")

def _build_sdk_impl(ctx):
    sdk = ctx.attr.sdk[SDKInfo]

    output_prefix = ctx.attr.name

    output_sdk = ctx.actions.declare_file(output_prefix + ".tar.zst")
    output_log_file = ctx.actions.declare_file(output_prefix + ".log")

    args = ctx.actions.args()
    args.add_all([
        "--log",
        output_log_file,
        ctx.executable._build_sdk,
        "--board=" + ctx.attr.board,
        "--output",
        output_sdk,
    ])

    layer_inputs = (
        sdk.layers +
        ctx.attr.overlays[OverlaySetInfo].layers +
        ctx.files.extra_tarballs +
        ctx.files.portage_config
    )
    args.add_all(layer_inputs, format_each = "--layer=%s", expand_directories = False)

    ctx.actions.run(
        inputs = depset(layer_inputs),
        outputs = [output_sdk, output_log_file],
        executable = ctx.executable._action_wrapper,
        tools = [ctx.executable._build_sdk],
        arguments = [args],
        execution_requirements = {
            # Disable sandbox to avoid creating a symlink forest.
            # This does not affect hermeticity since ebuild runs in a container.
            "no-sandbox": "",
            # Send SIGTERM instead of SIGKILL on user interruption.
            "supports-graceful-termination": "",
        },
        mnemonic = "BuildSDK",
    )

    return [
        DefaultInfo(files = depset([output_sdk])),
        OutputGroupInfo(
            logs = depset([output_log_file]),
        ),
        SDKInfo(
            layers = [output_sdk],
        ),
    ]

build_sdk = rule(
    implementation = _build_sdk_impl,
    attrs = {
        "board": attr.string(
            mandatory = True,
            doc = """
            The board name of the target SDK board.
            """,
        ),
        "extra_tarballs": attr.label_list(
            allow_files = True,
        ),
        "overlays": attr.label(
            providers = [OverlaySetInfo],
            mandatory = True,
        ),
        "portage_config": attr.label_list(
            providers = [PackageArtifactInfo],
            doc = """
            The portage config for the host and the target board. This should
            at minimum contain a make.conf file.
            """,
            mandatory = True,
        ),
        "progress_message": attr.string(
            doc = """
            Progress message for this target.
            If the message contains `{dep_count}' it will be replaced with the
            total number of dependencies that need to be installed.
            """,
            default = "Building %{label} with {dep_count} packages",
        ),
        "sdk": attr.label(
            doc = """
            The SDK that was used to create the packages listed in target_deps.
            """,
            mandatory = True,
            providers = [SDKInfo],
        ),
        "_action_wrapper": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/action_wrapper"),
        ),
        "_build_sdk": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/build_sdk"),
        ),
    },
)
