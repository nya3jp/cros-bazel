# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/ebuild/private/common/mountsdk:mountsdk.bzl", "COMMON_ATTRS", "MountSDKInfo", "debuggable_mountsdk", "mountsdk_generic")
load("//bazel/ebuild/private:common.bzl", "EbuildLibraryInfo")
load("//bazel/ebuild/private:interface_lib.bzl", "add_interface_library_args", "generate_interface_libraries")
load("@bazel_skylib//rules:common_settings.bzl", "BuildSettingInfo", "string_flag")

def _ebuild_impl(ctx):
    src_basename = ctx.file.ebuild.basename.rsplit(".", 1)[0]

    output_binary_package_file = ctx.actions.declare_file(src_basename + ".tbz2")
    output_log_file = ctx.actions.declare_file(src_basename + ".log")

    args = ctx.actions.args()

    ebuild_inside_path = ctx.file.ebuild.path.removeprefix(
        ctx.file.ebuild.owner.workspace_root + "/",
    ).removeprefix("internal/overlays/")
    args.add("--ebuild=%s=%s" % (ebuild_inside_path, ctx.file.ebuild.path))

    args.add_all(ctx.files.git_trees, format_each = "--git-tree=%s")

    add_interface_library_args(
        input_targets = ctx.attr.shared_lib_deps,
        args = args,
    )

    prebuilt = ctx.attr.prebuilt[BuildSettingInfo].value
    build_providers = mountsdk_generic(
        ctx,
        progress_message_name = ctx.file.ebuild.basename,
        inputs = [ctx.file.ebuild] + ctx.files.git_trees,
        binpkg_output_file = output_binary_package_file,
        outputs = [output_binary_package_file, output_log_file],
        args = args,
        log_output_file = output_log_file,
        generate_run_action = not prebuilt,
    )

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

    return [
        DefaultInfo(files = depset(
            [output_binary_package_file, output_log_file] +
            interface_library_outputs,
        )),
    ] + build_providers + interface_library_providers

_ebuild = rule(
    implementation = _ebuild_impl,
    attrs = dict(
        ebuild = attr.label(
            mandatory = True,
            allow_single_file = [".ebuild"],
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
        **COMMON_ATTRS
    ),
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

    debuggable_mountsdk(
        name = name,
        orig_rule = _ebuild,
        prebuilt = ":%s_prebuilt" % name,
        **kwargs
    )

    _ebuild_test_run(
        name = name + "_test",
        target = name,
        visibility = kwargs.get("visibility", None),
    )
