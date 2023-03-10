# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/ebuild/private:common.bzl", "BinaryPackageInfo", "EbuildLibraryInfo", "SDKInfo", "relative_path_in_package", "single_binary_package_set_info")
load("//bazel/ebuild/private:interface_lib.bzl", "add_interface_library_args", "generate_interface_libraries")
load("//rules_cros/toolchains/bash:defs.bzl", "BASH_RUNFILES_ATTR", "wrap_binary_with_args")
load("@bazel_skylib//rules:common_settings.bzl", "BuildSettingInfo")

def _format_file_arg(file):
    return "--file=%s=%s" % (relative_path_in_package(file), file.path)

# Attributes common to the `ebuild`/`ebuild_debug`/`ebuild_test` rule.
_EBUILD_COMMON_ATTRS = dict(
    ebuild = attr.label(
        mandatory = True,
        allow_single_file = [".ebuild"],
    ),
    distfiles = attr.label_keyed_string_dict(
        allow_files = True,
    ),
    srcs = attr.label_list(
        doc = "src files used by the ebuild",
        allow_files = True,
    ),
    git_trees = attr.label_list(
        doc = """
        The git tree objects listed in the CROS_WORKON_TREE variable.
        """,
        allow_empty = True,
        allow_files = True,
    ),
    files = attr.label_list(
        allow_files = True,
    ),
    runtime_deps = attr.label_list(
        providers = [BinaryPackageInfo],
    ),
    shared_lib_deps = attr.label_list(
        doc = """
        The shared libraries this target will link against.
        """,
        providers = [EbuildLibraryInfo],
    ),
    allow_network_access = attr.bool(
        default = False,
        doc = """
        Allows the build process to access the network. This should be True only
        when the package explicitly requests network access, e.g.
        RESTRICT=network-sandbox.
        """,
    ),
    sdk = attr.label(
        providers = [SDKInfo],
        mandatory = True,
    ),
    _action_wrapper = attr.label(
        executable = True,
        cfg = "exec",
        default = Label("//bazel/ebuild/private/cmd/action_wrapper"),
    ),
    _install_deps = attr.label(
        executable = True,
        cfg = "exec",
        default = Label("//bazel/ebuild/private/cmd/install_deps"),
    ),
    _build_package = attr.label(
        executable = True,
        cfg = "exec",
        default = Label("//bazel/ebuild/private/cmd/build_package"),
    ),
)

def _compute_build_package_args(ctx, output_path):
    """
    Computes the arguments to pass to build_package.

    This function can be called only from `ebuild`, `ebuild_debug`, and
    `ebuild_test`. Particularly, the current rule must include
    _EBUILD_COMMON_ATTRS in its attribute definition.

    Args:
        ctx: ctx: A context objected passed to the rule implementation.
        output_path: Optional[str]: A file path where an output binary package
            file is saved. If None, a binary package file is not saved.

    Returns:
        (args, inputs) where:
            args: Args: Arguments to pass to build_package.
            inputs: Depset[File]: Inputs to build_package.
    """
    args = ctx.actions.args()
    direct_inputs = []
    transitive_inputs = []

    # Basic arguments
    sdk = ctx.attr.sdk[SDKInfo]
    args.add("--board=" + sdk.board)
    if output_path:
        args.add("--output=" + output_path)

    # --ebuild
    ebuild_inside_path = ctx.file.ebuild.path.removeprefix(
        ctx.file.ebuild.owner.workspace_root + "/",
    ).removeprefix("internal/packages/")
    args.add("--ebuild=%s=%s" % (ebuild_inside_path, ctx.file.ebuild.path))

    # --file
    for file in ctx.attr.files:
        args.add_all(file.files, map_each = _format_file_arg)
        transitive_inputs.append(file.files)

    # --distfile
    for distfile, distfile_name in ctx.attr.distfiles.items():
        files = distfile.files.to_list()
        if len(files) != 1:
            fail("cannot refer to multi-file rule in distfiles")
        file = files[0]
        args.add("--distfile=%s=%s" % (distfile_name, file.path))
        direct_inputs.append(file)

    # --layer for SDK
    args.add_all(sdk.layers, format_each = "--layer=%s", expand_directories = False)
    direct_inputs.extend(sdk.layers)

    # --layer for overlays
    for overlay in sdk.overlays.overlays:
        args.add("--layer=%s" % overlay.file.path)
        direct_inputs.append(overlay.file)

    # --layer for source code
    for file in ctx.files.srcs:
        args.add("--layer=%s" % file.path)
        direct_inputs.append(file)

    # --git-tree
    args.add_all(ctx.files.git_trees, format_each = "--git-tree=%s")
    direct_inputs.extend(ctx.files.git_trees)

    # --allow-network-access
    if ctx.attr.allow_network_access:
        args.add("--allow-network-access")

    # Consume interface libraries.
    interface_library_inputs = add_interface_library_args(
        input_targets = ctx.attr.shared_lib_deps,
        args = args,
    )
    transitive_inputs.append(interface_library_inputs)

    inputs = depset(direct_inputs, transitive = transitive_inputs)
    return args, inputs

def _ebuild_impl(ctx):
    src_basename = ctx.file.ebuild.basename.rsplit(".", 1)[0]

    # Declare outputs.
    output_binary_package_file = ctx.actions.declare_file(src_basename + ".tbz2")
    output_log_file = ctx.actions.declare_file(src_basename + ".log")

    # Compute arguments and inputs to build_package.
    args, inputs = _compute_build_package_args(ctx, output_path = output_binary_package_file.path)

    # Define the main action.
    prebuilt = ctx.attr.prebuilt[BuildSettingInfo].value
    if prebuilt:
        gsutil_path = ctx.attr._gsutil_path[BuildSettingInfo].value
        ctx.actions.run(
            inputs = [],
            outputs = [output_binary_package_file],
            executable = ctx.executable._download_prebuilt,
            arguments = [gsutil_path, prebuilt, output_binary_package_file.path],
            execution_requirements = {
                "requires-network": "",
                "no-sandbox": "",
                "no-remote": "",
            },
            progress_message = "Downloading %s" % prebuilt,
        )
        ctx.actions.write(output_log_file, "Downloaded from %s\n" % prebuilt)
    else:
        ctx.actions.run(
            inputs = inputs,
            outputs = [output_binary_package_file, output_log_file],
            executable = ctx.executable._action_wrapper,
            tools = [ctx.executable._build_package],
            arguments = ["--output", output_log_file.path, ctx.executable._build_package.path, args],
            execution_requirements = {
                # Send SIGTERM instead of SIGKILL on user interruption.
                "supports-graceful-termination": "",
                # Disable sandbox to avoid creating a symlink forest.
                # This does not affect hermeticity since ebuild runs in a container.
                "no-sandbox": "",
                "no-remote": "",
            },
            mnemonic = "Ebuild",
            progress_message = "Building " + ctx.file.ebuild.basename,
        )

    # Generate interface libraries.
    interface_library_outputs, interface_library_providers = generate_interface_libraries(
        ctx = ctx,
        input_binary_package_file = output_binary_package_file,
        output_base_dir = src_basename,
        headers = ctx.attr.headers,
        pkg_configs = ctx.attr.pkg_configs,
        shared_libs = ctx.attr.shared_libs,
        static_libs = ctx.attr.static_libs,
        extract_interface_executable = ctx.executable._extract_interface,
    )

    # Compute provider data.
    direct_runtime_deps = tuple([
        target[BinaryPackageInfo]
        for target in ctx.attr.runtime_deps
    ])
    transitive_runtime_deps = depset(
        direct_runtime_deps,
        transitive = [
            pkg.transitive_runtime_deps
            for pkg in direct_runtime_deps
        ],
        order = "postorder",
    )
    all_files = depset(
        [output_binary_package_file],
        transitive = [pkg.all_files for pkg in direct_runtime_deps],
        order = "postorder",
    )
    package_info = BinaryPackageInfo(
        file = output_binary_package_file,
        all_files = all_files,
        direct_runtime_deps = direct_runtime_deps,
        transitive_runtime_deps = transitive_runtime_deps,
    )
    package_set_info = single_binary_package_set_info(package_info)
    return [
        DefaultInfo(files = depset(
            [output_binary_package_file, output_log_file] +
            interface_library_outputs,
        )),
        package_info,
        package_set_info,
    ] + interface_library_providers

ebuild = rule(
    implementation = _ebuild_impl,
    doc = "Builds a Portage binary package from an ebuild file.",
    attrs = dict(
        headers = attr.string_list(
            allow_empty = True,
            doc = """
            The path inside the binpkg that contains the public C headers
            exported by this library.
            """,
        ),
        pkg_configs = attr.string_list(
            allow_empty = True,
            doc = """
            The path inside the binpkg that contains the pkg-config
            (man 1 pkg-config) `pc` files exported by this package.
            The `pc` is used to look up the CFLAGS and LDFLAGS required to link
            to the library.
            """,
        ),
        shared_libs = attr.string_list(
            allow_empty = True,
            doc = """
            The path inside the binpkg that contains shared object libraries.
            """,
        ),
        static_libs = attr.string_list(
            allow_empty = True,
            doc = """
            The path inside the binpkg that contains static libraries.
            """,
        ),
        prebuilt = attr.label(providers = [BuildSettingInfo]),
        _extract_interface = attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/extract_interface"),
        ),
        _download_prebuilt = attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private:download_prebuilt"),
        ),
        _gsutil_path = attr.label(
            providers = [BuildSettingInfo],
            default = Label("//bazel/ebuild/private:gsutil_path"),
        ),
        **_EBUILD_COMMON_ATTRS
    ),
)

_DEBUG_SCRIPT = """
# Arguments passed in during build time are passed in relative to the execroot,
# which means all files passed in are relative paths starting with bazel-out/
# Thus, we cd to the directory in our working directory containing a bazel-out.

wd="$(pwd)"
cd "${wd%%/bazel-out/*}"

# The runfiles manifest file contains relative paths, which are evaluated
# relative to the working directory. Since we provide our own working directory,
# we need to use the RUNFILES_DIR instead.
export RUNFILES_DIR="${RUNFILES_MANIFEST_FILE%_manifest}"
unset RUNFILES_MANIFEST_FILE
"""

def _ebuild_debug_impl(ctx):
    src_basename = ctx.file.ebuild.basename.rsplit(".", 1)[0]

    # Declare outputs.
    output_debug_script = ctx.actions.declare_file(src_basename + "_debug.sh")

    # Compute arguments and inputs to build_package.
    args, inputs = _compute_build_package_args(ctx, output_path = None)
    return wrap_binary_with_args(
        ctx,
        out = output_debug_script,
        binary = ctx.attr._build_package,
        args = args,
        content_prefix = _DEBUG_SCRIPT,
        runfiles = ctx.runfiles(transitive_files = inputs),
    )

ebuild_debug = rule(
    implementation = _ebuild_debug_impl,
    executable = True,
    doc = "Enters the ephemeral chroot to build a Portage binary package in.",
    attrs = dict(
        _bash_runfiles = BASH_RUNFILES_ATTR,
        **_EBUILD_COMMON_ATTRS
    ),
)

def _ebuild_test_impl(ctx):
    src_basename = ctx.file.ebuild.basename.rsplit(".", 1)[0]

    # Declare outputs.
    output_runner_script = ctx.actions.declare_file(src_basename + "_test.sh")

    # Compute arguments and inputs to build_package.
    args, inputs = _compute_build_package_args(ctx, output_path = None)
    args.add("--test")

    return wrap_binary_with_args(
        ctx,
        out = output_runner_script,
        binary = ctx.attr._build_package,
        args = args,
        runfiles = ctx.runfiles(transitive_files = inputs),
    )

ebuild_test = rule(
    implementation = _ebuild_test_impl,
    doc = "Runs ebuild tests.",
    attrs = dict(
        _bash_runfiles = BASH_RUNFILES_ATTR,
        **_EBUILD_COMMON_ATTRS
    ),
    test = True,
)
