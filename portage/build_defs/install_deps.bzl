# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "SDKLayer", "sdk_to_layer_list")

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

def install_deps(
        ctx,
        output_prefix,
        board,
        sdk,
        overlays,
        portage_configs,
        install_set,
        executable_action_wrapper,
        executable_fast_install_packages,
        progress_message,
        contents):
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
        portage_configs: list[File]: Tarballs containing portage config.
        install_set: Depset[BinaryPackageInfo]: Binary package targets to
            install. This depset must be closed over transitive runtime
            dependencies; that is, if the depset contains a package X, it must
            also contain all transitive dependencies of the package X. Also,
            This depset's to_list() must return packages in a valid installation
            order, i.e. a package's runtime dependencies are fully satisfied by
            packages that appear before it.
        executable_action_wrapper: File: An executable file of action_wrapper.
        executable_fast_install_packages: File: An executable file of
            fast_install_packages.
        progress_message: str: Progress message for the installation action.
        contents: str: Defines the types of layers to return. Valid options are:
            full, sparse, or interface. When interface is set, it has the same
            effect as sparse, but it also adds the `interface_file` to the
            SDKLayer.

    Returns:
        struct where:
            layers: list[SDKLayer]: Layers used to build the SDK.
            log_file: File: Log file generated when building the layers.
            trace_file: File: Trace file generated when building the layers.
    """
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
            if package.partial.path != conflicting_package.partial.path:
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
        "--log",
        output_log_file,
        "--profile",
        output_profile_file,
        "--temp-dir",
        output_log_file.dirname + "/tmp",
        executable_fast_install_packages,
    ])
    args.add("--root-dir=%s" % sysroot)

    if contents in ["sparse", "interface"]:
        args.add("--sparse-vdb")

    input_layers = sdk_to_layer_list(sdk) + overlays.layers + portage_configs
    args.add_all(
        input_layers,
        format_each = "--layer=%s",
        expand_directories = False,
    )
    inputs = input_layers[:]

    layers = []

    for i, package in enumerate(install_list):
        package_output_prefix = "%s.%d" % (output_prefix, i)
        output_preinst = ctx.actions.declare_directory(
            "%s.preinst" % package_output_prefix,
        )
        output_postinst = ctx.actions.declare_directory(
            "%s.postinst" % package_output_prefix,
        )

        if contents == "full":
            installed_layer = package.contents.full.installed
            staged_layer = package.contents.full.staged
            interface_layer = None
        else:
            installed_layer = package.contents.internal.installed
            staged_layer = package.contents.internal.staged

            if contents == "interface":
                interface_layer = package.contents.internal.interface
            else:
                interface_layer = None

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
        args.add_joined(
            "--install",
            [
                package.partial,
                installed_layer,
                staged_layer,
                output_preinst,
                output_postinst,
            ],
            join_with = ",",
            expand_directories = False,
        )

        inputs.extend([
            package.partial,
            installed_layer,
            staged_layer,
        ])

        outputs.extend([output_preinst, output_postinst])

        layers.extend([
            SDKLayer(file = output_preinst),
            SDKLayer(file = installed_layer, interface_file = interface_layer),
            SDKLayer(file = output_postinst),
        ])

    actual_progress_message = progress_message.replace(
        "{dep_count}",
        str(len(install_list)),
    )

    ctx.actions.run(
        inputs = depset(inputs),
        outputs = outputs,
        executable = executable_action_wrapper,
        tools = [executable_fast_install_packages],
        arguments = [args],
        execution_requirements = {
            # Disable sandbox to avoid creating a symlink forest.
            # This does not affect hermeticity since ebuild runs in a container.
            "no-sandbox": "",
            # Send SIGTERM instead of SIGKILL on user interruption.
            "supports-graceful-termination": "",
        },
        mnemonic = "InstallDeps",
        progress_message = actual_progress_message,
    )

    return struct(
        layers = layers,
        log_file = output_log_file,
        trace_file = output_profile_file,
    )
