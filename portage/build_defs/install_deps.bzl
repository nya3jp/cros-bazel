# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("install_groups.bzl", "calculate_install_groups", "map_install_group")

def _compute_slot_key(package):
    """
    Computes a slot key from BinaryPackageInfo.

    A slot key is a string that uniquely describes a package slot occuppied by a
    package, in the form "<category>/<package-name>:<main-slot>@<sysroot>".
    Two packages with the same slot key can't be installed at the same time on
    a system.

    Args:
        package: BinaryPackageInfo: Describes a package.

    Returns:
        A slot key.
    """
    main_slot = package.slot.split("/")[0]
    return "%s/%s:%s@%s" % (
        package.category,
        package.package_name,
        main_slot,
        package.contents.sysroot,
    )

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
    sysroot = "/build/%s" % board if board else "/"

    slot_key_to_package = {}  # dict[str, BinaryPackageInfo]

    # Inspect already installed packages.
    for package in sdk.packages.to_list():
        slot_key = _compute_slot_key(package)
        conflicting_package = slot_key_to_package.get(slot_key)
        if conflicting_package:
            fail(
                ("Slot conflict: cannot install %s when %s has been already " +
                 "installed with the same slot key %s") % (
                    package.file.path,
                    conflicting_package.file.path,
                    slot_key,
                ),
            )
        else:
            slot_key_to_package[slot_key] = package

    # Create a list of packages to install.
    install_list = []
    for package in install_set.to_list():
        slot_key = _compute_slot_key(package)
        conflicting_package = slot_key_to_package.get(slot_key)
        if conflicting_package:
            # Skip identical packages.
            if package.file.path != conflicting_package.file.path:
                fail(
                    ("Slot conflict: cannot install %s and %s that have " +
                     "the same slot key %s") % (
                        package.file.path,
                        conflicting_package.file.path,
                        slot_key,
                    ),
                )
        else:
            install_list.append(package)
            slot_key_to_package[slot_key] = package

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
    args.add("--root-dir=%s" % sysroot)

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

        if package.contents.sysroot != sysroot:
            fail(
                ("Requested to install %s/%s-%s to %s, but its installed " +
                 "contents layers were generated only for %s") % (
                    package.category,
                    package.package_name,
                    package.version,
                    sysroot,
                    package.contents.sysroot,
                ),
            )
        args.add("--install=%s,%s,%s,%s,%s" % (
            package.file.path,
            package.contents.installed.path,
            package.contents.staged.path,
            output_preinst.path,
            output_postinst.path,
        ))

        inputs.extend([
            package.file,
            package.contents.installed,
            package.contents.staged,
        ])
        outputs.extend([output_preinst, output_postinst])
        new_layers.extend([
            output_preinst,
            package.contents.installed,
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
        board: str: The target board name to install dependencies for. If it is
            non-empty, packages are installed to the corresponding sysroot
            (ROOT="/build/<board>"). If it is an empty string, packages are
            installed to the host (ROOT="/").
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
