# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load(":common.bzl", "SDKInfo")

def _sdk_to_archive_impl(ctx):
    output_prefix = ctx.attr.name
    output_tarball = ctx.actions.declare_file(output_prefix + ".tar.zst")
    output_log = ctx.actions.declare_file(output_prefix + ".log")
    output_profile = ctx.actions.declare_file(output_prefix + ".profile.json")

    args = ctx.actions.args()
    args.add_all([
        "--log",
        output_log,
        "--profile",
        output_profile,
        ctx.executable._sdk_to_archive,
        "--output",
        output_tarball,
    ], expand_directories = False)

    sdk = ctx.attr.sdk[SDKInfo]
    args.add_all(
        sdk.layers,
        before_each = "--layer",
        expand_directories = False,
    )

    inputs = [ctx.executable._sdk_to_archive] + sdk.layers
    outputs = [output_tarball, output_log, output_profile]

    ctx.actions.run(
        inputs = inputs,
        outputs = outputs,
        executable = ctx.executable._action_wrapper,
        tools = [ctx.executable._sdk_to_archive],
        arguments = [args],
        mnemonic = "SdkToArchive",
        progress_message = "Creating SDK tarball %{output}",
        execution_requirements = {
            # Needed so we can use durable trees.
            "no-sandbox": "",
            # Send SIGTERM instead of SIGKILL on user interruption.
            "supports-graceful-termination": "",
        },
    )

    return [
        DefaultInfo(files = depset([output_tarball])),
        OutputGroupInfo(
            logs = depset([output_log]),
            traces = depset([output_profile]),
        ),
    ]

sdk_to_archive = rule(
    implementation = _sdk_to_archive_impl,
    attrs = {
        "sdk": attr.label(
            providers = [SDKInfo],
            mandatory = True,
        ),
        "_action_wrapper": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/action_wrapper"),
        ),
        "_sdk_to_archive": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/sdk_to_archive"),
        ),
    },
)
