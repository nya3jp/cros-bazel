# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/ebuild/private:common.bzl", "BinaryPackageInfo", "EbuildLibraryInfo", "MountSDKInfo", "SDKInfo", "relative_path_in_package")
load("//bazel/ebuild/private:install_deps.bzl", "install_deps")
load("//bazel/ebuild/private:interface_lib.bzl", "add_interface_library_args", "generate_interface_libraries")
load("@bazel_skylib//rules:common_settings.bzl", "BuildSettingInfo", "string_flag")

def _format_file_arg(file):
    return "--file=%s=%s" % (relative_path_in_package(file), file.path)

def _ebuild_impl(ctx):
    src_basename = ctx.file.ebuild.basename.rsplit(".", 1)[0]

    # Declare outputs.
    output_binary_package_file = ctx.actions.declare_file(src_basename + ".tbz2")
    output_log_file = ctx.actions.declare_file(src_basename + ".log")

    # Variables to record arguments and inputs of the action.
    args = ctx.actions.args()
    direct_inputs = []
    transitive_inputs = []

    # Basic arguments
    sdk = ctx.attr.sdk[SDKInfo]
    args.add_all([
        "--output=" + output_binary_package_file.path,
        "--board=" + sdk.board,
    ])

    # --ebuild
    ebuild_inside_path = ctx.file.ebuild.path.removeprefix(
        ctx.file.ebuild.owner.workspace_root + "/",
    ).removeprefix("internal/overlays/")
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

    # --layer for dependencies
    transitive_build_time_deps_targets = depset(
        ctx.attr.build_deps,
        # Pull in runtime dependencies of build-time dependencies.
        # TODO: Revisit this logic to see if we can avoid pulling in transitive
        # dependencies.
        transitive = [dep[BinaryPackageInfo].transitive_runtime_deps_targets for dep in ctx.attr.build_deps],
        order = "postorder",
    )
    deps_layers = install_deps(
        ctx = ctx,
        output_prefix = src_basename + "-deps",
        sdk = ctx.attr.sdk[SDKInfo],
        install_targets = transitive_build_time_deps_targets,
        executable_action_wrapper = ctx.executable._action_wrapper,
        executable_install_deps = ctx.executable._install_deps,
        progress_message = "Installing dependencies for %s" % ctx.file.ebuild.basename,
    )
    args.add_all(deps_layers, format_each = "--layer=%s", expand_directories = False)
    direct_inputs.extend(deps_layers)

    # Consume interface libraries.
    add_interface_library_args(
        input_targets = ctx.attr.shared_lib_deps,
        args = args,
    )

    # Now we've fully decided arguments and inputs.
    # Do not modify direct_inputs and transitive_inputs after this.
    inputs = depset(direct_inputs, transitive = transitive_inputs)

    # Define an action to build the package.
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
            tools = [ctx.executable._builder],
            arguments = ["--output", output_log_file.path, ctx.executable._builder.path, args],
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
    transitive_runtime_deps_files = depset(
        [output_binary_package_file],
        transitive = [dep[BinaryPackageInfo].transitive_runtime_deps_files for dep in ctx.attr.runtime_deps],
        order = "postorder",
    )
    transitive_runtime_deps_targets = depset(
        ctx.attr.runtime_deps,
        transitive = [dep[BinaryPackageInfo].transitive_runtime_deps_targets for dep in ctx.attr.runtime_deps],
        order = "postorder",
    )
    return [
        DefaultInfo(files = depset(
            [output_binary_package_file, output_log_file] +
            interface_library_outputs,
        )),
        BinaryPackageInfo(
            file = output_binary_package_file,
            transitive_runtime_deps_files = transitive_runtime_deps_files,
            transitive_runtime_deps_targets = transitive_runtime_deps_targets,
            direct_runtime_deps_targets = ctx.attr.runtime_deps,
        ),
        MountSDKInfo(
            executable = ctx.executable._builder,
            executable_runfiles = ctx.attr._builder[DefaultInfo].default_runfiles,
            args = args,
            direct_inputs = direct_inputs,
            transitive_inputs = transitive_inputs,
        ),
    ] + interface_library_providers

_ebuild = rule(
    implementation = _ebuild_impl,
    attrs = dict(
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
        build_deps = attr.label_list(
            providers = [BinaryPackageInfo],
        ),
        runtime_deps = attr.label_list(
            providers = [BinaryPackageInfo],
        ),
        files = attr.label_list(
            allow_files = True,
        ),
        sdk = attr.label(
            providers = [SDKInfo],
            mandatory = True,
        ),
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
        shared_lib_deps = attr.label_list(
            doc = """
            The shared libraries this target will link against.
            """,
            providers = [EbuildLibraryInfo],
        ),
        git_trees = attr.label_list(
            doc = """
            The git tree objects listed in the CROS_WORKON_TREE variable.
            """,
            allow_empty = True,
            allow_files = True,
        ),
        prebuilt = attr.label(providers = [BuildSettingInfo]),
        _action_wrapper = attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/action_wrapper"),
        ),
        _builder = attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/build_package"),
        ),
        _install_deps = attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/install_deps"),
        ),
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
    ),
)

def _ebuild_debug_impl(ctx):
    info = ctx.attr.target[MountSDKInfo]

    wrapper = ctx.actions.declare_file(ctx.label.name)

    args = ctx.actions.args()
    args.add_all([wrapper, info.executable])
    ctx.actions.run(
        executable = ctx.executable._create_debug_script,
        arguments = [args, info.args],
        outputs = [wrapper],
    )

    runfiles = ctx.runfiles(transitive_files = depset(info.direct_inputs, transitive = info.transitive_inputs))
    runfiles = runfiles.merge_all([
        info.executable_runfiles,
        ctx.attr._bash_runfiles[DefaultInfo].default_runfiles,
    ])

    return [DefaultInfo(
        files = depset([wrapper]),
        runfiles = runfiles,
        executable = wrapper,
    )]

_ebuild_debug = rule(
    implementation = _ebuild_debug_impl,
    attrs = dict(
        target = attr.label(
            providers = [MountSDKInfo],
            mandatory = True,
        ),
        _bash_runfiles = attr.label(default = "@bazel_tools//tools/bash/runfiles"),
        _create_debug_script = attr.label(
            default = "//bazel/ebuild/private/common/mountsdk:create_debug_file",
            executable = True,
            cfg = "exec",
        ),
    ),
    executable = True,
)

def _ebuild_test_impl(ctx):
    info = ctx.attr.target[MountSDKInfo]

    test_runner = ctx.actions.declare_file(ctx.label.name)

    args = ctx.actions.args()
    args.add_all([test_runner, info.executable])
    ctx.actions.run(
        executable = ctx.executable._generate_test_runner,
        arguments = [args, info.args, "--test"],
        outputs = [test_runner],
    )

    runfiles = ctx.runfiles(transitive_files = depset(info.direct_inputs, transitive = info.transitive_inputs))
    runfiles.merge(info.executable_runfiles)

    return [DefaultInfo(
        files = depset([test_runner]),
        runfiles = runfiles,
        executable = test_runner,
    )]

# TODO(b/269558613) Rename this to _ebuild_test.
# A rule name can end with "_test" only when test = True.
_ebuild_test_run = rule(
    implementation = _ebuild_test_impl,
    attrs = dict(
        target = attr.label(
            providers = [MountSDKInfo],
            mandatory = True,
        ),
        _generate_test_runner = attr.label(
            default = ":generate_test_runner",
            executable = True,
            cfg = "exec",
        ),
    ),
    # TODO(b/269558613) Change this to "test = True".
    executable = True,
)

def ebuild(name, **kwargs):
    string_flag(
        name = name + "_prebuilt",
        build_setting_default = "",
    )
    _ebuild(
        name = name,
        prebuilt = ":%s_prebuilt" % name,
        **kwargs
    )
    _ebuild_debug(
        name = name + "_debug",
        target = name,
        visibility = kwargs.get("visibility", None),
    )
    _ebuild_test_run(
        name = name + "_test",
        target = name,
        visibility = kwargs.get("visibility", None),
    )
