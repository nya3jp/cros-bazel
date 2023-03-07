# Copyright 2023 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/ebuild/private:common.bzl", "BinaryPackageSetInfo", "SDKInfo")

def _build_image_impl(ctx):
    # Declare outputs.
    output_image_file = ctx.actions.declare_file(ctx.attr.name + ".bin")
    output_log_file = ctx.actions.declare_file(ctx.attr.name + ".log")

    # Compute arguments and inputs to build_image.
    args = ctx.actions.args()
    direct_inputs = []
    transitive_inputs = []

    sdk = ctx.attr.sdk[SDKInfo]
    args.add_all([
        "--output=" + output_image_file.path,
        "--board=" + sdk.board,
    ])

    args.add_all(sdk.layers, format_each = "--layer=%s", expand_directories = False)
    direct_inputs.extend(sdk.layers)

    for overlay in sdk.overlays.overlays:
        args.add("--layer=%s" % overlay.file.path)
        direct_inputs.append(overlay.file)

    args.add_all(ctx.files.files, format_each = "--layer=%s", expand_directories = False)
    direct_inputs.extend(ctx.files.files)

    target_package_files = depset(
        transitive = [
            packages[BinaryPackageSetInfo].files
            for packages in ctx.attr.target_packages
        ],
    )
    args.add_all(target_package_files, format_each = "--target-package=%s")
    transitive_inputs.append(target_package_files)

    host_package_files = depset(
        transitive = [
            packages[BinaryPackageSetInfo].files
            for packages in ctx.attr.host_packages
        ],
    )
    args.add_all(host_package_files, format_each = "--host-package=%s")
    transitive_inputs.append(target_package_files)

    if ctx.attr.override_base_packages:
        args.add_all(ctx.attr.override_base_packages, format_each = "--override-base-package=%s")

    inputs = depset(direct_inputs, transitive = transitive_inputs)

    # Define the main action.
    ctx.actions.run(
        inputs = inputs,
        outputs = [output_image_file, output_log_file],
        executable = ctx.executable._action_wrapper,
        tools = [ctx.executable._build_image],
        arguments = ["--output", output_log_file.path, ctx.executable._build_image.path, args],
        execution_requirements = {
            # Send SIGTERM instead of SIGKILL on user interruption.
            "supports-graceful-termination": "",
            # Disable sandbox to avoid creating a symlink forest.
            # This does not affect hermeticity since ebuild runs in a container.
            "no-sandbox": "",
            "no-remote": "",
        },
        progress_message = "Building " + output_image_file.basename,
    )

    return [DefaultInfo(files = depset([output_image_file, output_log_file]))]

build_image = rule(
    implementation = _build_image_impl,
    doc = "Builds a ChromeOS image.",
    attrs = dict(
        target_packages = attr.label_list(
            providers = [BinaryPackageSetInfo],
            mandatory = True,
            doc = """
            Packages included in the image.
            """,
        ),
        host_packages = attr.label_list(
            providers = [BinaryPackageSetInfo],
            allow_empty = True,
            doc = """
            Host binary packages required by chromite's build_image script.
            """,
        ),
        override_base_packages = attr.string_list(
            allow_empty = True,
            doc = """
            Overrides packages to install on the base image. If empty,
            virtual/target-os is selected.
            """,
        ),
        files = attr.label_list(
            allow_files = True,
            doc = """
            Extra files to be made available in the ephemeral chroot.
            """,
        ),
        sdk = attr.label(
            providers = [SDKInfo],
            mandatory = True,
        ),
        _action_wrapper = attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/action_wrapper"),
        ),
        _build_image = attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/build_image"),
        ),
    ),
)
