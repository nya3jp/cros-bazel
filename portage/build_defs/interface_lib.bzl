# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
Implements generating and consuming interface libraries.

An interface library is a minimal representation of a library whose
implementation details are omitted. For example, we can generate an interface
library from a shared library object (.so) by trimming everything that are not
need by the linker, e.g. TEXT section in ELF. Interface libraries allow us to
avoid package rebuilds when a library package is updated without changing its
external interfaces.
"""

load("@bazel_skylib//lib:paths.bzl", "paths")
load("//bazel/portage/build_defs:common.bzl", "EbuildLibraryInfo", "compute_input_file_path")

def _format_input_file_arg(strip_prefix, file, use_runfiles):
    return "--sysroot-file=%s=%s" % (file.path.removeprefix(strip_prefix), compute_input_file_path(file, use_runfiles))

def add_interface_library_args(input_targets, args, use_runfiles):
    """
    Computes the arguments to pass to build_package to link interface libraries.

    A build rule that consumes interface libraries should call this function
    to get an input depset and arguments to pass to the program.

    Args:
        input_targets: list[Target]: A list of targets that provide interface
            libraries. Typically it is from the shared_lib_deps attribute.
        args: Args: An Args object where necessary arguments are added in order
            to depend on the interface libraries.
        use_runfiles: bool: Whether to refer to input file paths in relative to
            execroot or runfiles directory. See compute_input_file_path for
            details.

    Returns:
        Depset[File]: A depset representing interface library inputs.
    """
    depsets = []

    for input_target in input_targets:
        lib_info = input_target[EbuildLibraryInfo]
        deps = depset(transitive = [lib_info.headers, lib_info.pkg_configs, lib_info.shared_libs])

        args.add_all(
            deps,
            allow_closure = True,
            map_each = lambda file: _format_input_file_arg(lib_info.strip_prefix, file, use_runfiles),
        )
        depsets.append(deps)

    return depset(transitive = depsets)

def _declare_interface_library_outputs(
        ctx,
        input_paths,
        allowed_extensions,
        output_base_dir,
        args):
    """
    Declares interface library outputs.

    Args:
        ctx: ctx: A context object passed to the rule implementation function.
        input_paths: list[str]: A list of interface library file paths contained
            in the binary package.
        allowed_extensions: Optional[list[str]]: An optional list of allowed
            extensions in interface libraries. This function will fail if the
            given interface library file paths contain non-allowed extensions.
        output_base_dir: str: The relative directory where interface library
            outputs are saved.
        args: Args: An Args object where necessary arguments are added in order
            to extract interface libraries.

    Returns:
        list[File]: A list of files declared as outputs of the current rule.
    """
    outputs = []
    for input_path in input_paths:
        if not paths.is_absolute(input_path):
            fail("%s is not absolute" % input_path)
        file_name = paths.basename(input_path)

        _, extension = paths.split_extension(file_name)
        if allowed_extensions and extension not in allowed_extensions:
            fail("%s must be of file type %s, got %s" % (input_path, allowed_extensions, extension))

        file = ctx.actions.declare_file(paths.join(output_base_dir, input_path[1:]))
        outputs.append(file)
        args.add_joined(
            "--output_file"[input_path, file],
            join_with = "=",
        )

    return outputs

def generate_interface_libraries(
        ctx,
        input_binary_package_file,
        output_base_dir,
        headers,
        pkg_configs,
        shared_libs,
        static_libs,
        extract_interface_executable,
        action_wrapper_executable):
    """
    Declares an action to generate interface libraries from a binary package.

    A build rule that generates interface libraries should call this function
    to get a list of interface library outputs and associated providers.

    Args:
        ctx: ctx: A context object passed to the rule implementation function.
        input_binary_package_file: File: An input binary package file to extract
            interface libraries from.
        output_base_dir: str: The base directory where extracted interface
            libraries are saved.
        headers: list[str]: A list of header files in the binary package that
            makes interface libraries. They must be absolute file paths.
        pkg_configs: list[str]: A list of pkg-config files (.pc) in the binary
            package that makes interface libraries. They must be absolute file
            paths.
        shared_libs: list[str]: A list of shared library files (.so) in the
            binary package that makes interface libraries. They must be absolute
            file paths.
        static_libs: list[str]: A list of static library files (.a) in the
            binary package that makes interface libraries. They must be absolute
            file paths.
        extract_interface_executable: File: The "extract_interface" executable
            file.
        action_wrapper_executable: File: The "action_wrapper" executable file.

    Returns:
        (outputs, providers) where:
            outputs: list[File]: Generated interface library files.
            providers: list[Provider]: A list of providers that should be
                attached to the current build target.
    """
    output_log = ctx.actions.declare_file(output_base_dir + ".extract_interface.log")

    args = ctx.actions.args()
    args.add_all([
        "--log",
        output_log,
        # TODO: Enable profiling.
        extract_interface_executable,
        "--binpkg",
        input_binary_package_file,
    ])

    xpak_outputs = []
    for xpak_file in ["NEEDED", "REQUIRES", "PROVIDES", "USE"]:
        file = ctx.actions.declare_file(paths.join(output_base_dir, "xpak", xpak_file))
        xpak_outputs.append(file)
        args.add_joined(
            "--xpak",
            [xpak_file, file],
            join_with = "=?",
        )

    files_output_base_dir = paths.join(output_base_dir, "files")
    header_outputs = _declare_interface_library_outputs(
        ctx = ctx,
        input_paths = headers,
        allowed_extensions = [".h", ".hpp"],
        output_base_dir = files_output_base_dir,
        args = args,
    )
    pkg_config_outputs = _declare_interface_library_outputs(
        ctx = ctx,
        input_paths = pkg_configs,
        allowed_extensions = [".pc"],
        output_base_dir = files_output_base_dir,
        args = args,
    )
    shared_lib_outputs = _declare_interface_library_outputs(
        ctx = ctx,
        input_paths = shared_libs,
        allowed_extensions = None,
        output_base_dir = files_output_base_dir,
        args = args,
    )
    static_lib_outputs = _declare_interface_library_outputs(
        ctx = ctx,
        input_paths = static_libs,
        allowed_extensions = [".a"],
        output_base_dir = files_output_base_dir,
        args = args,
    )

    files_outputs = header_outputs + pkg_config_outputs + shared_lib_outputs + static_lib_outputs
    outputs = xpak_outputs + files_outputs + [output_log]

    ctx.actions.run(
        inputs = [input_binary_package_file],
        outputs = outputs,
        executable = action_wrapper_executable,
        tools = [extract_interface_executable],
        arguments = [args],
        progress_message = "Generating interface libraries for %s" % paths.basename(input_binary_package_file.path),
    )

    providers = [
        # TODO: Only generate EbuildLibraryInfo if we have files specified.
        EbuildLibraryInfo(
            # TODO: Fix the computation of strip_prefix. Currently this assumes
            # that output_base_dir is exactly the same as the stem part of the
            # binary package file name.
            strip_prefix = paths.join(input_binary_package_file.path.rsplit(".", 1)[0], "files"),
            headers = depset(header_outputs),
            pkg_configs = depset(pkg_config_outputs),
            shared_libs = depset(shared_lib_outputs),
            static_libs = depset(static_lib_outputs),
        ),
        # TODO: Create a CCInfo provider so we can start using rules_cc.
    ]

    return outputs, providers
