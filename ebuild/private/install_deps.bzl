# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def _map_install_group(group):
    """
    Computes an --install-target argument for an install group.

    Args:
        group: list[BinaryPackageInfo]: An install group.

    Returns:
        str: A value for the --install-target flag.
    """
    return ":".join([pkg.file.path for pkg in group])

def _calculate_install_groups(install_list):
    """
    Splits a package set to install groups.

    Args:
        install_list: list[BinaryPackageInfo]: A list of packages to install.
            This list must be closed over transitive runtime dependencies.

    Returns:
        list[list[BinaryPackageInfo]]: An ordered list containing a list of
            packages that can be installed in parallel.
    """
    groups = []
    remaining_packages = install_list[:]
    seen = {}

    for _ in range(100):
        if len(remaining_packages) == 0:
            break

        satisfied_list = []
        not_satisfied_list = []
        for package in remaining_packages:
            all_seen = True
            for dep in package.direct_runtime_deps:
                if dep.file.path not in seen:
                    all_seen = False
                    break

            if all_seen:
                satisfied_list.append(package)
            else:
                not_satisfied_list.append(package)

        if len(satisfied_list) == 0:
            fail("Dependency list is unsatisfiable")

        for dep in satisfied_list:
            seen[dep.file.path] = True

        groups.append(satisfied_list)
        remaining_packages = not_satisfied_list

    if len(remaining_packages) > 0:
        fail("Too many dependencies")

    return groups

def install_deps(
        ctx,
        output_prefix,
        board,
        sdk,
        overlays,
        install_set,
        executable_action_wrapper,
        executable_install_deps,
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
            also contain all transitive dependencies of the package X.
        executable_action_wrapper: File: An executable file of action_wrapper.
        executable_install_deps: File: An executable file of install_deps.
        progress_message: str: Progress message for the installation action.

    Returns:
        list[File]: Files representing file system layers.
    """
    output_root = ctx.actions.declare_directory(output_prefix)
    output_log_file = ctx.actions.declare_file(output_prefix + ".log")

    args = ctx.actions.args()
    args.add_all([
        "--output=" + output_log_file.path,
        executable_install_deps.path,
        "--output=" + output_root.path,
    ])
    if board:
        args.add("--board=" + board)

    # TODO: Can we avoid the costly to_list() operation?
    install_list = install_set.to_list()
    direct_inputs = [pkg.file for pkg in install_list]

    layer_inputs = sdk.layers + overlays.layers
    args.add_all(layer_inputs, format_each = "--layer=%s", expand_directories = False)
    direct_inputs.extend(layer_inputs)

    install_groups = _calculate_install_groups(install_list)
    args.add_all(install_groups, map_each = _map_install_group, format_each = "--install-target=%s")

    ctx.actions.run(
        inputs = depset(direct_inputs),
        outputs = [output_root, output_log_file],
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

    return [output_root]
