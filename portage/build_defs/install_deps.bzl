# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("install_groups.bzl", "calculate_install_groups", "map_install_group")

def install_deps(
        ctx,
        output_prefix,
        board,
        sdk,
        overlays,
        install_set,
        executable_action_wrapper,
        executable_install_deps,
        progress_message,
        use_layers):
    """
    Creates an action which builds file system layers in which the build dependencies are installed.

    Args:
        ctx: ctx: A context object passed to the rule implementation.
        output_prefix: str: A file name prefix to prepend to output files
            defined in this function.
        board: Option[str]: The target board name to install dependencies for. If None
            then the packages are installed into the host sysroot.
        sdk: SDKInfo: The provider describing the base file system layers.
        overlays: OverlaySetInfo: Overlays providing packages.
        install_set: Depset[BinaryPackageInfo]: Binary package targets to
            install. This depset must be closed over transitive runtime
            dependencies; that is, if the depset contains a package X, it must
            also contain all transitive dependencies of the package X. Also,
            This depset's to_list() must return packages in a valid installation
            order, i.e. a package's runtime dependencies are fully satisfied by
            packages that appear before it.
        executable_action_wrapper: File: An executable file of action_wrapper.
        executable_install_deps: File: An executable file of install_deps.
        progress_message: str: Progress message for the installation action.
        use_layers: bool: Use package layers when possible, otherwise perform
            a binpkg installation.

    Returns:
        list[File]: Files representing file system layers.
    """
    output_root = ctx.actions.declare_directory(output_prefix)
    output_log_file = ctx.actions.declare_file(output_prefix + ".log")
    output_profile_file = ctx.actions.declare_file(
        output_prefix + ".profile.json",
    )

    args = ctx.actions.args()
    args.add_all([
        "--log=" + output_log_file.path,
        "--profile=" + output_profile_file.path,
        executable_install_deps.path,
        "--output=" + output_root.path,
    ])
    if board:
        args.add("--board=" + board)

    install_list = install_set.to_list()

    install_tuple = calculate_install_groups(
        install_list,
        provided_packages = sdk.packages,
        use_layers = use_layers,
    )

    if use_layers:
        install_groups, install_layers = install_tuple
    else:
        install_groups, install_layers = install_tuple, []

    args.add_all(install_groups, map_each = map_install_group, format_each = "--install-target=%s")

    direct_inputs = []
    for group in install_groups:
        for package in group:
            direct_inputs.append(package.file)

    layer_inputs = sdk.layers + overlays.layers + install_layers
    args.add_all(layer_inputs, format_each = "--layer=%s", expand_directories = False)
    direct_inputs.extend(layer_inputs)

    progress_message = progress_message.replace(
        "{dep_count}",
        str(len(install_list)),
    )

    progress_message = progress_message.replace(
        "{cached_count}",
        str(len(install_layers)),
    )

    ctx.actions.run(
        inputs = depset(direct_inputs),
        outputs = [output_root, output_log_file, output_profile_file],
        executable = executable_action_wrapper,
        tools = [executable_install_deps],
        arguments = [args],
        execution_requirements = {
            # No need to cache the extracted binpkgs.
            "no-remote-cache": "",
            # Send SIGTERM instead of SIGKILL on user interruption.
            "supports-graceful-termination": "",
            # Disable sandbox to avoid creating a symlink forest.
            # This does not affect hermeticity since ebuild runs in a container.
            "no-sandbox": "",
        },
        mnemonic = "InstallDeps",
        progress_message = progress_message,
    )

    return install_layers + [output_root]
