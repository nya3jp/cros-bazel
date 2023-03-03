# Copyright 2023 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/ebuild/private:common.bzl", "BinaryPackageInfo")

def _map_install_group(targets):
    files = []
    for target in targets:
        file = target[BinaryPackageInfo].file
        files.append(file.path)
    return ":".join(files)

def _calculate_install_groups(build_deps):
    seen = {}

    # An ordered list containing a list of deps that can be installed in parallel
    levels = []

    remaining_targets = build_deps.to_list()

    for _ in range(100):
        if len(remaining_targets) == 0:
            break

        satisfied_list = []
        not_satisfied_list = []
        for target in remaining_targets:
            info = target[BinaryPackageInfo]

            all_seen = True
            for runtime_target in info.direct_runtime_deps_targets:
                if not seen.get(runtime_target.label):
                    all_seen = False
                    break

            if all_seen:
                satisfied_list.append(target)
            else:
                not_satisfied_list.append(target)

        if len(satisfied_list) == 0:
            fail("Dependency list is unsatisfiable")

        for target in satisfied_list:
            seen[target.label] = True

        levels.append(satisfied_list)
        remaining_targets = not_satisfied_list

    if len(remaining_targets) > 0:
        fail("Too many dependencies")

    return levels

def install_deps(
        ctx,
        output_prefix,
        sdk,
        install_targets,
        executable_action_wrapper,
        executable_install_deps,
        progress_message):
    """
    Creates an action which builds file system layers in which the build dependencies are installed.

    Args:
        ctx: ctx: A context object passed to the rule implementation.
        output_prefix: str: A file name prefix to prepend to output files
            defined in this function.
        sdk: SDKInfo: The provider describing the base file system layers.
        install_targets: Depset[Target]: Binary package targets to install.
            This depset must be closed over transitive runtime dependencies;
            that is, if the depset contains a package X, it must also contain
            all transitive dependencies of the package X.
        executable_action_wrapper: File: An executable file of action_wrapper.
        executable_install_deps: File: An executable file of install_deps.
        progress_message: str: Progress message for the installation action.

    Returns:
        list[File]: Files representing file system layers.
    """
    output_directory = ctx.actions.declare_directory(output_prefix)
    output_tarball = ctx.actions.declare_file(output_prefix + "-symlinks.tar")
    output_log_file = ctx.actions.declare_file(output_prefix + ".log")

    args = ctx.actions.args()
    args.add_all([
        "--output=" + output_log_file.path,
        executable_install_deps.path,
        "--board=" + sdk.board,
        "--output-dir=" + output_directory.path,
        "--output-symlink-tar=" + output_tarball.path,
    ])

    direct_inputs = [target[BinaryPackageInfo].file for target in install_targets.to_list()]

    args.add_all(sdk.layers, format_each = "--layer=%s", expand_directories = False)
    direct_inputs.extend(sdk.layers)

    for overlay in sdk.overlays.overlays:
        args.add("--layer=%s" % overlay.file.path)
        direct_inputs.append(overlay.file)

    install_groups = _calculate_install_groups(install_targets)
    args.add_all(install_groups, map_each = _map_install_group, format_each = "--install-target=%s")

    ctx.actions.run(
        inputs = depset(direct_inputs),
        outputs = [output_directory, output_tarball, output_log_file],
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

    return [output_tarball, output_directory]
