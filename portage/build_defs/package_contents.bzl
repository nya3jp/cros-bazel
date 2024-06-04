# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "ContentLayerTypesInfo", "ContentLayersInfo", "sdk_to_layer_list")

def _generate_contents_layer(
        ctx,
        binary_package,
        output_name,
        image_prefix,
        vdb_prefix,
        host,
        sparse_vdb,
        drop_revision,
        executable_action_wrapper,
        executable_extract_package):
    output_contents_dir = ctx.actions.declare_directory(output_name)
    output_log = ctx.actions.declare_file(output_name + ".log")

    arguments = ctx.actions.args()
    arguments.add_all([
        "--log",
        output_log,
        "--temp-dir",
        output_log.dirname + "/tmp",
        executable_extract_package,
        "--input-binary-package",
        binary_package,
        "--output-directory",
        output_contents_dir,
        "--image-prefix=" + image_prefix,
        "--vdb-prefix=" + vdb_prefix,
    ], expand_directories = False)

    if sparse_vdb:
        arguments.add("--sparse-vdb")

    if drop_revision:
        arguments.add("--drop-revision")

    if host:
        arguments.add("--host")

    ctx.actions.run(
        inputs = [binary_package],
        outputs = [output_contents_dir, output_log],
        executable = executable_action_wrapper,
        tools = [executable_extract_package],
        arguments = [arguments],
        execution_requirements = {
            # Disable sandbox to avoid creating a symlink forest.
            # This does not affect hermeticity since ebuild runs in a container.
            "no-sandbox": "",
            # Send SIGTERM instead of SIGKILL on user interruption.
            "supports-graceful-termination": "",
        },
        progress_message = "Extracting %{label}",
    )

    return output_contents_dir

def _generate_interface_layer(
        ctx,
        base_sdk,
        sysroot,
        input,
        output_name,
        interface_library_allowlist,
        executable_action_wrapper,
        executable_create_interface_layer):
    output_contents_dir = ctx.actions.declare_directory(output_name)
    output_log = ctx.actions.declare_file(output_name + ".log")

    arguments = ctx.actions.args()
    arguments.add_all([
        "--log",
        output_log,
        "--temp-dir",
        output_log.dirname + "/tmp",
        executable_create_interface_layer,
        "--sysroot",
        sysroot,
        "--input",
        input,
        "--output",
        output_contents_dir,
    ], expand_directories = False)

    layers = sdk_to_layer_list(base_sdk)

    arguments.add_all(
        layers,
        before_each = "--layer",
        expand_directories = False,
    )

    arguments.add_all(
        interface_library_allowlist,
        before_each = "--include",
        expand_directories = False,
    )

    ctx.actions.run(
        inputs = [input] + layers,
        outputs = [output_contents_dir, output_log],
        executable = executable_action_wrapper,
        tools = [executable_create_interface_layer],
        arguments = [arguments],
        execution_requirements = {
            # Disable sandbox to avoid creating a symlink forest.
            # This does not affect hermeticity since ebuild runs in a container.
            "no-sandbox": "",
            # Send SIGTERM instead of SIGKILL on user interruption.
            "supports-graceful-termination": "",
        },
        mnemonic = "GenerateInterfaceLibraryLayer",
        progress_message = "Generating interface library layer for %{label}",
    )

    return output_contents_dir

def generate_contents(
        ctx,
        base_sdk,
        binary_package,
        output_prefix,
        board,
        generate_interface_libraries,
        interface_library_allowlist,
        executable_action_wrapper,
        executable_extract_package,
        executable_create_interface_layer):
    """
    Defines actions that build contents layers from a binary package.

    Args:
        ctx: ctx: A context object passed to the rule implementation.
        base_sdk: Optional[SDKInfo]: SDK used to generate interface libraries.
            This SDK should omit any target specific packages to avoid excessive
            dependencies.
        binary_package: File: Binary package file.
        output_prefix: str: A file name prefix to prepend to output directories
            defined in this function.
        board: str: The target board name to install a package for. If it is
            non-empty, the package is to be installed to the corresponding
            sysroot (ROOT="/build/<board>"). If it is an empty string, the
            package is to be installed to the host (ROOT="/").
        generate_interface_libraries: bool: Generate an interface layer.
        interface_library_allowlist: List[str]: A list of files that will be
            included in the interface layer.
        executable_action_wrapper: File: An executable file of action_wrapper.
        executable_extract_package: File: An executable file of extract_package.
        executable_create_interface_layer: File: An executable file of create_interface_layer.

    Returns:
        ContentLayerTypesInfo: A struct suitable to set in
            BinaryPackageInfo.contents.
    """
    if board:
        name = board
        sysroot = "/build/%s" % board
        host = board == "amd64-host"
    else:
        name = "host"
        sysroot = "/"
        host = True

    prefix = sysroot.lstrip("/")

    internal_installed = _generate_contents_layer(
        ctx = ctx,
        binary_package = binary_package,
        output_name = "%s.%s.sparse.installed.contents" % (output_prefix, name),
        image_prefix = prefix,
        vdb_prefix = prefix,
        host = host,
        # We don't need most of the vdb once the package has been installed.
        sparse_vdb = True,
        drop_revision = True,
        executable_action_wrapper = executable_action_wrapper,
        executable_extract_package = executable_extract_package,
    )

    internal_interface = None
    if base_sdk and generate_interface_libraries:
        internal_interface = _generate_interface_layer(
            ctx = ctx,
            base_sdk = base_sdk,
            sysroot = sysroot,
            input = internal_installed,
            output_name = "%s.%s.interface.contents" % (output_prefix, name),
            interface_library_allowlist = interface_library_allowlist,
            executable_action_wrapper = executable_action_wrapper,
            executable_create_interface_layer = executable_create_interface_layer,
        )

    return ContentLayerTypesInfo(
        sysroot = sysroot,
        full = ContentLayersInfo(
            installed = _generate_contents_layer(
                ctx = ctx,
                binary_package = binary_package,
                output_name = "%s.%s.full.installed.contents" % (output_prefix, name),
                image_prefix = prefix,
                vdb_prefix = prefix,
                host = host,
                sparse_vdb = False,
                drop_revision = False,
                executable_action_wrapper = executable_action_wrapper,
                executable_extract_package = executable_extract_package,
            ),
            staged = _generate_contents_layer(
                ctx = ctx,
                binary_package = binary_package,
                output_name = "%s.%s.full.staged.contents" % (output_prefix, name),
                image_prefix = ".image",
                vdb_prefix = prefix,
                host = host,
                sparse_vdb = False,
                drop_revision = False,
                executable_action_wrapper = executable_action_wrapper,
                executable_extract_package = executable_extract_package,
            ),
            interface = None,
        ),
        internal = ContentLayersInfo(
            installed = internal_installed,
            staged = _generate_contents_layer(
                ctx = ctx,
                binary_package = binary_package,
                output_name = "%s.%s.sparse.staged.contents" % (output_prefix, name),
                image_prefix = ".image",
                vdb_prefix = prefix,
                host = host,
                # We need the full vdb so we can run the install hooks.
                sparse_vdb = False,
                drop_revision = True,
                executable_action_wrapper = executable_action_wrapper,
                executable_extract_package = executable_extract_package,
            ),
            interface = internal_interface,
        ),
    )
