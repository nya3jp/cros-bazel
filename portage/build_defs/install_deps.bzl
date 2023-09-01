# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("install_groups.bzl", "calculate_install_groups", "map_install_group")

def _fast_install_packages(
        ctx,
        output_prefix,
        board,
        sdk,
        overlays,
        install_set,
        executable_action_wrapper,
        executable_fast_install_packages,
        progress_message):
    # Skip already installed packages.
    installed = {}
    for package in sdk.packages.to_list():
        installed[package.file] = True
    install_list = [
        package
        for package in install_set.to_list()
        if not installed.get(package.file, False)
    ]

    output_log_file = ctx.actions.declare_file("%s.log" % output_prefix)
    output_profile_file = ctx.actions.declare_file(
        "%s.profile.json" % output_prefix,
    )
    outputs = [output_log_file, output_profile_file]

    args = ctx.actions.args()
    args.add_all([
        "--log=" + output_log_file.path,
        "--profile=" + output_profile_file.path,
        executable_fast_install_packages.path,
    ])
    if board:
        args.add("--root-dir=/build/%s" % board)
    else:
        args.add("--root-dir=/")

    input_layers = sdk.layers + overlays.layers
    args.add_all(
        input_layers,
        format_each = "--layer=%s",
        expand_directories = False,
    )
    inputs = input_layers[:]

    new_layers = []

    for i, package in enumerate(install_list):
        package_output_prefix = "%s.%d" % (output_prefix, i)
        output_preinst = ctx.actions.declare_directory(
            "%s.preinst" % package_output_prefix,
        )
        output_postinst = ctx.actions.declare_directory(
            "%s.postinst" % package_output_prefix,
        )

        contents_info = getattr(package.contents, board or "__host__")
        args.add("--install=%s,%s,%s,%s,%s" % (
            package.file.path,
            contents_info.installed.path,
            contents_info.staged.path,
            output_preinst.path,
            output_postinst.path,
        ))

        inputs.extend([
            package.file,
            contents_info.installed,
            contents_info.staged,
        ])
        outputs.extend([output_preinst, output_postinst])
        new_layers.extend([
            output_preinst,
            contents_info.installed,
            output_postinst,
        ])

    actual_progress_message = progress_message.replace(
        "{dep_count}",
        str(len(install_list)),
    ).replace(
        "{cached_count}",
        str(len(install_list)),
    )

    ctx.actions.run(
        inputs = depset(inputs),
        outputs = outputs,
        executable = executable_action_wrapper,
        tools = [executable_fast_install_packages],
        arguments = [args],
        execution_requirements = {
            # Send SIGTERM instead of SIGKILL on user interruption.
            "supports-graceful-termination": "",
            # Disable sandbox to avoid creating a symlink forest.
            # This does not affect hermeticity since ebuild runs in a container.
            "no-sandbox": "",
        },
        mnemonic = "InstallDeps",
        progress_message = actual_progress_message,
    )

    return new_layers

def install_deps(
        ctx,
        output_prefix,
        board,
        sdk,
        overlays,
        install_set,
        strategy,
        executable_action_wrapper,
        executable_install_deps,
        executable_fast_install_packages,
        progress_message):
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
        strategy: str: Specifies the strategy to install packages. Valid values
            are:
                "fast": Uses installed contents layers to fully avoid copying
                    package contents.
                "naive": Similar to "fast" but uses installed contents layers
                    only for packages without install hooks.
                "slow": Simply uses emerge to install packages into a single
                    layer.
        executable_action_wrapper: File: An executable file of action_wrapper.
        executable_install_deps: File: An executable file of install_deps.
        executable_fast_install_packages: File: An executable file of
            fast_install_packages.
        progress_message: str: Progress message for the installation action.

    Returns:
        list[File]: Files representing file system layers.
    """
    if strategy == "fast":
        return _fast_install_packages(
            ctx = ctx,
            output_prefix = output_prefix,
            board = board,
            sdk = sdk,
            overlays = overlays,
            install_set = install_set,
            executable_action_wrapper = executable_action_wrapper,
            executable_fast_install_packages = executable_fast_install_packages,
            progress_message = progress_message,
        )

    output_root = ctx.actions.declare_directory(output_prefix)
    output_log_file = ctx.actions.declare_file(output_prefix + ".log")
    output_profile_file = ctx.actions.declare_file(
        output_prefix + ".profile.json",
    )

    use_layers = strategy == "naive"

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
