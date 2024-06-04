# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@rules_pkg//pkg:providers.bzl", "PackageArtifactInfo")
load(":common.bzl", "BinaryPackageInfo", "BinaryPackageSetInfo", "OverlaySetInfo", "SDKInfo", "SDKLayer", "sdk_to_layer_list")
load(":install_deps.bzl", "install_deps")

def _sdk_from_archive_impl(ctx):
    output_prefix = ctx.attr.out or ctx.attr.name
    output_root = ctx.actions.declare_directory(output_prefix)
    output_log = ctx.actions.declare_file(output_prefix + ".log")
    output_profile = ctx.actions.declare_file(output_prefix + ".profile.json")

    args = ctx.actions.args()
    args.add_all([
        "--log",
        output_log,
        "--profile",
        output_profile,
        "--temp-dir",
        output_log.dirname + "/tmp",
        ctx.executable._sdk_from_archive,
        "--input",
        ctx.file.src,
        "--output",
        output_root,
    ], expand_directories = False)

    inputs = [ctx.executable._sdk_from_archive, ctx.file.src]
    outputs = [output_root, output_log, output_profile]

    ctx.actions.run(
        inputs = inputs,
        outputs = outputs,
        executable = ctx.executable._action_wrapper,
        tools = [ctx.executable._sdk_from_archive],
        arguments = [args],
        mnemonic = "SdkFromArchive",
        progress_message = ctx.attr.progress_message,
    )

    return [
        DefaultInfo(files = depset([output_root])),
        OutputGroupInfo(
            logs = depset([output_log]),
            traces = depset([output_profile]),
        ),
        SDKInfo(
            layers = [
                SDKLayer(file = output_root),
            ],
            packages = depset(),
        ),
    ]

sdk_from_archive = rule(
    implementation = _sdk_from_archive_impl,
    attrs = {
        "out": attr.string(
            doc = "Output directory name. Defaults to the target name.",
        ),
        "progress_message": attr.string(
            default = "Extracting SDK archive",
        ),
        "src": attr.label(
            mandatory = True,
            allow_single_file = True,
        ),
        "_action_wrapper": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/action_wrapper"),
        ),
        "_sdk_from_archive": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/sdk_from_archive"),
        ),
    },
)

def _sdk_update_impl(ctx):
    output_prefix = ctx.attr.out or ctx.attr.name
    output_root = ctx.actions.declare_directory(output_prefix)
    output_log = ctx.actions.declare_file(output_prefix + ".log")
    output_profile = ctx.actions.declare_file(output_prefix + ".profile.json")

    args = ctx.actions.args()
    args.add_all([
        "--log",
        output_log,
        "--profile",
        output_profile,
        "--temp-dir",
        output_log.dirname + "/tmp",
        ctx.executable._sdk_update,
        "--output",
        output_root,
    ], expand_directories = False)

    base_sdk = ctx.attr.base[SDKInfo]
    layer_inputs = sdk_to_layer_list(base_sdk)
    args.add_all(layer_inputs, format_each = "--layer=%s", expand_directories = False)

    args.add_all(ctx.files.extra_tarballs, format_each = "--install-tarball=%s")

    inputs = depset(
        [ctx.executable._sdk_update] + layer_inputs + ctx.files.extra_tarballs,
    )

    outputs = [output_root, output_log, output_profile]

    ctx.actions.run(
        inputs = inputs,
        outputs = outputs,
        executable = ctx.executable._action_wrapper,
        tools = [ctx.executable._sdk_update],
        arguments = [args],
        execution_requirements = {
            # Disable sandbox to avoid creating a symlink forest.
            # This does not affect hermeticity since sdk_update runs in a container.
            "no-sandbox": "",
            # Send SIGTERM instead of SIGKILL on user interruption.
            "supports-graceful-termination": "",
        },
        mnemonic = "SdkUpdate",
        progress_message = "Building %{label}",
    )

    return [
        DefaultInfo(files = depset([output_root])),
        OutputGroupInfo(
            logs = depset([output_log]),
            traces = depset([output_profile]),
        ),
        SDKInfo(
            layers = base_sdk.layers + [
                SDKLayer(file = output_root),
            ],
            packages = base_sdk.packages,
        ),
    ]

sdk_update = rule(
    implementation = _sdk_update_impl,
    attrs = {
        "base": attr.label(
            mandatory = True,
            providers = [SDKInfo],
        ),
        "extra_tarballs": attr.label_list(
            allow_files = True,
        ),
        "out": attr.string(
            doc = "Output directory name. Defaults to the target name.",
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
            default = Label("//bazel/portage/bin/action_wrapper"),
        ),
        "_sdk_update": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/sdk_update"),
        ),
    },
)

def _sdk_install_deps_impl(ctx):
    sdk = ctx.attr.base[SDKInfo]

    install_set = depset(
        transitive = [dep[BinaryPackageSetInfo].packages for dep in ctx.attr.target_deps],
        order = "postorder",
    )

    deps = install_deps(
        ctx = ctx,
        output_prefix = ctx.attr.out or ctx.attr.name,
        board = ctx.attr.board,
        sdk = sdk,
        overlays = ctx.attr.overlays[OverlaySetInfo],
        portage_configs = ctx.files.portage_config,
        install_set = install_set,
        executable_action_wrapper = ctx.executable._action_wrapper,
        executable_fast_install_packages =
            ctx.executable._fast_install_packages,
        progress_message = ctx.attr.progress_message,
        contents = ctx.attr.contents,
    )

    return [
        DefaultInfo(files = depset(
            [layer.file for layer in deps.layers],
        )),
        OutputGroupInfo(
            logs = depset([deps.log_file]),
            traces = depset([deps.trace_file]),
        ),
        SDKInfo(
            layers = sdk.layers + deps.layers,
            packages = depset(transitive = [sdk.packages, install_set]),
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
            doc = """
            If set, the packages are installed into the board's sysroot,
            otherwise they are installed into the host's sysroot.
            """,
        ),
        "contents": attr.string(
            doc = """
            Specifies the how complete the dependency layers are.

            Valid options:
            * full: The layers will contain a full vdb and complete contents.
                Use this when calling emerge or exporting the layers in an
                image or tarball.
            * sparse: The layers will contain a sparse vdb that is only suitable
                for handling `has_version`. It contain all files in their
                original state.
            * interface: When set, it has the same effect as `sparse`, but in
                addition it will also enable interface library layers for the
                specified packages. An interface library layer is a layer with a
                sparse vdb and interface shared objects (shared objects without
                any code). All binaries and any non-essential files have also
                been removed. This layer should only be used for building
                dynamically linked libraries.

                An ebuild can choose to build with interface layers by setting
                `use_interface_libraries=True`.
            """,
            default = "sparse",
            values = ["full", "sparse", "interface"],
        ),
        "out": attr.string(
            doc = "Output directory name. Defaults to the target name.",
        ),
        "overlays": attr.label(
            providers = [OverlaySetInfo],
            mandatory = True,
        ),
        "portage_config": attr.label_list(
            providers = [PackageArtifactInfo],
            doc = """
            The portage config for the host and optionally the target. This should
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
            default = "Installing {dep_count} packages into %{label}",
        ),
        "target_deps": attr.label_list(
            doc = """
            Target packages to install in the SDK.
            """,
            providers = [BinaryPackageSetInfo],
        ),
        "_action_wrapper": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/action_wrapper"),
        ),
        "_fast_install_packages": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/fast_install_packages"),
        ),
    },
)

def _sdk_extend_impl(ctx):
    sdk = ctx.attr.base[SDKInfo]

    sdk = SDKInfo(
        layers = sdk.layers + [
            SDKLayer(file = layer)
            for layer in ctx.files.extra_tarballs
        ],
        packages = sdk.packages,
    )

    return [
        # We don't return any files since this rule doesn't create any.
        DefaultInfo(),
        sdk,
    ]

sdk_extend = rule(
    doc = "Adds extra tarballs to the SDK",
    implementation = _sdk_extend_impl,
    provides = [SDKInfo],
    attrs = {
        "base": attr.label(
            doc = """
            Base SDK to derive a new SDK from.
            """,
            mandatory = True,
            providers = [SDKInfo],
        ),
        "extra_tarballs": attr.label_list(
            allow_files = True,
            mandatory = True,
            doc = """
            Extra files to layer onto the base SDK.
            """,
        ),
    },
)

def _sdk_install_glibc_impl(ctx):
    output_prefix = ctx.attr.out or ctx.attr.name
    output_root = ctx.actions.declare_directory(output_prefix)
    output_log = ctx.actions.declare_file(output_prefix + ".log")
    output_profile = ctx.actions.declare_file(output_prefix + ".profile.json")

    args = ctx.actions.args()
    args.add_all([
        "--log",
        output_log,
        "--profile",
        output_profile,
        "--temp-dir",
        output_log.dirname + "/tmp",
        ctx.executable._sdk_install_glibc,
        "--output",
        output_root,
        "--board",
        ctx.attr.board,
    ], expand_directories = False)

    base_sdk = ctx.attr.base[SDKInfo]
    layer_inputs = sdk_to_layer_list(base_sdk)
    args.add_all(layer_inputs, format_each = "--layer=%s", expand_directories = False)

    glibc = ctx.attr.glibc[BinaryPackageInfo].partial
    args.add("--glibc", glibc)

    inputs = depset(
        [ctx.executable._sdk_install_glibc] + layer_inputs + [glibc],
    )

    outputs = [output_root, output_log, output_profile]

    ctx.actions.run(
        inputs = inputs,
        outputs = outputs,
        executable = ctx.executable._action_wrapper,
        tools = [ctx.executable._sdk_install_glibc],
        arguments = [args],
        execution_requirements = {
            # Disable sandbox to avoid creating a symlink forest.
            # This does not affect hermeticity since sdk_install_glibc runs in a container.
            "no-sandbox": "",
            # Send SIGTERM instead of SIGKILL on user interruption.
            "supports-graceful-termination": "",
        },
        mnemonic = "SdkInstallGlibc",
        progress_message = "Installing cross glibc into %{label}",
    )

    return [
        DefaultInfo(files = depset([output_root])),
        OutputGroupInfo(
            logs = depset([output_log]),
            traces = depset([output_profile]),
        ),
        SDKInfo(
            layers = base_sdk.layers + [
                SDKLayer(file = output_root),
            ],
            packages = depset(
                [ctx.attr.glibc[BinaryPackageInfo]],
                transitive = [base_sdk.packages],
            ),
        ),
    ]

sdk_install_glibc = rule(
    implementation = _sdk_install_glibc_impl,
    attrs = {
        "base": attr.label(
            mandatory = True,
            providers = [SDKInfo],
        ),
        "board": attr.string(
            doc = "The cross-* package is installed into the board's sysroot.",
            mandatory = True,
        ),
        "glibc": attr.label(
            doc = "The cross-*-cros-linux-gnu/glibc package to install",
            mandatory = True,
            providers = [BinaryPackageInfo],
        ),
        "out": attr.string(
            doc = "Output directory name. Defaults to the target name.",
        ),
        "_action_wrapper": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/action_wrapper"),
        ),
        "_sdk_install_glibc": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/sdk_install_glibc"),
        ),
    },
)

def _remote_toolchain_inputs(ctx):
    output_prefix = ctx.attr.name
    output = ctx.actions.declare_file(output_prefix + ".tar.zst")
    output_log = ctx.actions.declare_file(output_prefix + ".log")
    output_profile = ctx.actions.declare_file(output_prefix + ".profile.json")

    args = ctx.actions.args()
    args.add_all([
        "--log",
        output_log,
        "--profile",
        output_profile,
        "--temp-dir",
        output_log.dirname + "/tmp",
        ctx.executable._generate_reclient_inputs,
        "--output",
        output,
    ], expand_directories = False)

    layer_inputs = (
        sdk_to_layer_list(ctx.attr.sdk[SDKInfo]) +
        [ctx.file._chromite_src]
    )
    args.add_all(layer_inputs, format_each = "--layer=%s", expand_directories = False)

    inputs = depset(
        [ctx.executable._generate_reclient_inputs] + layer_inputs,
    )

    outputs = [output, output_log, output_profile]

    ctx.actions.run(
        inputs = inputs,
        outputs = outputs,
        executable = ctx.executable._action_wrapper,
        tools = [ctx.executable._generate_reclient_inputs],
        arguments = [args],
        execution_requirements = {
            # Disable sandbox to avoid creating a symlink forest.
            # This does not affect hermeticity since generate_reclient_inputs runs in a container.
            "no-sandbox": "",
            # Send SIGTERM instead of SIGKILL on user interruption.
            "supports-graceful-termination": "",
        },
        mnemonic = "GenerateReclientInputs",
        progress_message = "Building %{label}",
    )

    return [
        DefaultInfo(files = depset([output])),
        OutputGroupInfo(
            logs = depset([output_log]),
            traces = depset([output_profile]),
        ),
    ]

remote_toolchain_inputs = rule(
    implementation = _remote_toolchain_inputs,
    attrs = {
        "sdk": attr.label(
            doc = "The SDK to generate the remote_toolchain_inputs file for.",
            mandatory = True,
            providers = [SDKInfo],
        ),
        "_action_wrapper": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/action_wrapper"),
        ),
        "_chromite_src": attr.label(
            default = Label("@chromite//:src"),
            allow_single_file = True,
        ),
        "_generate_reclient_inputs": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/generate_reclient_inputs"),
        ),
    },
)
