# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("common.bzl", "ContentsLayersInfo")

_HOST_BOARD = "amd64-host"
_HOST_KEY = "__host__"

def _generate_contents_layer(
        ctx,
        binary_package,
        output_name,
        image_prefix,
        vdb_prefix,
        host,
        executable_action_wrapper,
        executable_extract_package):
    output_contents_dir = ctx.actions.declare_directory(output_name)
    output_log = ctx.actions.declare_file(output_name + ".log")

    arguments = [
        "--log=" + output_log.path,
        executable_extract_package.path,
        "--input-binary-package=" + binary_package.path,
        "--output-directory=" + output_contents_dir.path,
        "--image-prefix=" + image_prefix,
        "--vdb-prefix=" + vdb_prefix,
    ]

    if host:
        arguments.append("--host")

    ctx.actions.run(
        inputs = [binary_package],
        outputs = [output_contents_dir, output_log],
        executable = executable_action_wrapper,
        tools = [executable_extract_package],
        arguments = arguments,
        execution_requirements = {
            # Send SIGTERM instead of SIGKILL on user interruption.
            "supports-graceful-termination": "",
            # Disable sandbox to avoid creating a symlink forest.
            # This does not affect hermeticity since ebuild runs in a container.
            "no-sandbox": "",
        },
        progress_message = "Extracting %{label}",
    )

    return output_contents_dir

def _generate_contents_layers(
        ctx,
        binary_package,
        output_prefix,
        board,
        executable_action_wrapper,
        executable_extract_package):
    # board is None when we're generating for /, instead of /build/$BOARD.
    # Note that board can be still "amd64-host" when we're generating for
    # /build/amd64-host.
    sysroot = "build/%s" % board if board else ""
    host = board == None or board == _HOST_BOARD
    name = board or "host"
    return ContentsLayersInfo(
        installed = _generate_contents_layer(
            ctx = ctx,
            binary_package = binary_package,
            output_name = "%s.%s.installed.contents" % (output_prefix, name),
            image_prefix = sysroot,
            vdb_prefix = sysroot,
            host = host,
            executable_action_wrapper = executable_action_wrapper,
            executable_extract_package = executable_extract_package,
        ),
        staged = _generate_contents_layer(
            ctx = ctx,
            binary_package = binary_package,
            output_name = "%s.%s.staged.contents" % (output_prefix, name),
            image_prefix = ".image",
            vdb_prefix = sysroot,
            host = host,
            executable_action_wrapper = executable_action_wrapper,
            executable_extract_package = executable_extract_package,
        ),
    )

def generate_contents(
        ctx,
        binary_package,
        output_prefix,
        board,
        executable_action_wrapper,
        executable_extract_package):
    """
    Defines actions that build contents layers from a binary package.

    Args:
        ctx: ctx: A context object passed to the rule implementation.
        binary_package: File: Binary package file.
        output_prefix: str: A file name prefix to prepend to output directories
            defined in this function.
        board: str: The target board name to install a package for. If it is
            non-empty, the package is to be installed to the corresponding
            sysroot (ROOT="/build/<board>"). If it is an empty string, the
            package is to be installed to the host (ROOT="/").
        executable_action_wrapper: File: An executable file of action_wrapper.
        executable_extract_package: File: An executable file of extract_package.

    Returns:
        struct[ContentsLayersInfo]: A struct suitable to set in
            BinaryPackageInfo.contents.
    """

    if not board:
        board = _HOST_BOARD

    contents = {}

    contents[board] = _generate_contents_layers(
        ctx = ctx,
        binary_package = binary_package,
        output_prefix = output_prefix,
        board = board,
        executable_action_wrapper = executable_action_wrapper,
        executable_extract_package = executable_extract_package,
    )

    # If the target board is amd64-host, also generate a host installation.
    if board == _HOST_BOARD:
        contents[_HOST_KEY] = _generate_contents_layers(
            ctx = ctx,
            binary_package = binary_package,
            output_prefix = output_prefix,
            board = None,
            executable_action_wrapper = executable_action_wrapper,
            executable_extract_package = executable_extract_package,
        )

    return struct(**contents)
