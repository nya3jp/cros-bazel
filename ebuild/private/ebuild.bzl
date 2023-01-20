# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/ebuild/private/common/mountsdk:mountsdk.bzl", "COMMON_ATTRS", "debuggable_mountsdk", "mountsdk_generic")
load("//bazel/ebuild/private:common.bzl", "BinaryPackageInfo", "EbuildLibraryInfo")
load("@bazel_skylib//lib:paths.bzl", "paths")
load("@bazel_skylib//rules:common_settings.bzl", "BuildSettingInfo", "string_flag")

def _format_input_file_arg(strip_prefix, file):
    return "--sysroot-file=%s=%s" % (file.path.removeprefix(strip_prefix), file.path)

def _ebuild_basename(ctx):
    return ctx.file.ebuild.basename.rsplit(".", 1)[0]

def _ebuild_declare_outputs(ctx, basename, args, paths, allowed_extensions = None):
    outputs = []
    for inside_path in paths:
        if not paths.is_absolute(inside_path):
            fail("%s is not absolute" % inside_path)
        file_name = paths.basename(inside_path)

        _, extension = paths.split_extension(file_name)
        if allowed_extensions and extension not in allowed_extensions:
            fail("%s must be of file type %s, got %s" % (inside_path, allowed_extensions, extension))

        file = ctx.actions.declare_file(paths.join(basename, "files", inside_path[1:]))
        outputs.append(file)
        args.add("--output-file=%s=%s" % (inside_path, file.path))

    return outputs

def _ebuild_calculate_inputs(ctx, args):
    inputs = []

    for shared_lib_dep in ctx.attr.shared_lib_deps:
        lib_info = shared_lib_dep[EbuildLibraryInfo]
        deps = depset(transitive = [lib_info.headers, lib_info.pkg_configs, lib_info.shared_libs])

        args.add_all(
            deps,
            allow_closure = True,
            map_each = lambda file: _format_input_file_arg(lib_info.strip_prefix, file),
        )

def _ebuild_impl(ctx):
    src_basename = _ebuild_basename(ctx)
    binpkg_output_file = ctx.actions.declare_file(src_basename + ".tbz2")

    ebuild_inside_path = ctx.file.ebuild.path.removeprefix(
        ctx.file.ebuild.owner.workspace_root + "/",
    ).removeprefix(
        "internal/overlays/",
    )
    args = ctx.actions.args()
    args.add("--ebuild=%s=%s" % (ebuild_inside_path, ctx.file.ebuild.path))

    _ebuild_calculate_inputs(ctx, args)

    extraction_args = ctx.actions.args()
    extraction_args.add("--binpkg=%s" % binpkg_output_file.path)
    xpak_outputs = []
    for xpak_file in ["NEEDED", "REQUIRES", "PROVIDES"]:
        file = ctx.actions.declare_file(paths.join(src_basename, "xpak", xpak_file))
        xpak_outputs.append(file)
        extraction_args.add("--xpak=%s=?%s" % (xpak_file, file.path))

    header_outputs = _ebuild_declare_outputs(
        ctx,
        src_basename,
        extraction_args,
        ctx.attr.headers,
        allowed_extensions = [".h", ".hpp"],
    )
    pkg_config_outputs = _ebuild_declare_outputs(
        ctx,
        src_basename,
        extraction_args,
        ctx.attr.pkg_config,
        allowed_extensions = [".pc"],
    )
    shared_lib_outputs = _ebuild_declare_outputs(ctx, src_basename, extraction_args, ctx.attr.shared_libs)
    static_lib_outputs = _ebuild_declare_outputs(
        ctx,
        src_basename,
        extraction_args,
        ctx.attr.static_libs,
        allowed_extensions = [".a"],
    )

    output_group_info = OutputGroupInfo(
        binpkg = depset([binpkg_output_file]),
        xpak = depset(xpak_outputs),
        headers = depset(header_outputs),
        pkg_configs = depset(pkg_config_outputs),
        shared_libs = depset(shared_lib_outputs),
        static_libs = depset(static_lib_outputs),
    )

    outputs = []
    outputs.extend(xpak_outputs)
    outputs.extend(header_outputs)
    outputs.extend(pkg_config_outputs)
    outputs.extend(shared_lib_outputs)
    outputs.extend(static_lib_outputs)

    ctx.actions.run(
        inputs = [binpkg_output_file],
        outputs = outputs,
        executable = ctx.executable._extract_interface,
        arguments = [extraction_args],
        progress_message = "Generating metadata for %s" % ctx.label.name,
    )

    # TODO: Only generate this if we have files specified
    library_info = EbuildLibraryInfo(
        strip_prefix = paths.join(binpkg_output_file.path.rsplit(".", 1)[0], "files"),
        headers = output_group_info.headers,
        pkg_configs = output_group_info.pkg_configs,
        shared_libs = output_group_info.shared_libs,
        static_libs = output_group_info.static_libs,
    )

    # TODO: Create a CCInfo provider so we can start using rules_cc
    providers = [
        output_group_info,
        library_info,
        DefaultInfo(files = depset(outputs + [binpkg_output_file])),
    ]

    prebuilt = ctx.attr.prebuilt[BuildSettingInfo].value
    providers.extend(mountsdk_generic(
        ctx,
        progress_message_name = ctx.file.ebuild.basename,
        inputs = [ctx.file.ebuild],
        binpkg_output_file = binpkg_output_file,
        outputs = [binpkg_output_file],
        args = args,
        install_deps = True,
        generate_run_action = not prebuilt,
    ))

    if prebuilt:
        gsutil_path = ctx.attr._gsutil_path[BuildSettingInfo].value
        ctx.actions.run(
            inputs = [],
            outputs = [binpkg_output_file],
            executable = ctx.executable._download_prebuilt,
            arguments = [gsutil_path, prebuilt, binpkg_output_file.path],
            execution_requirements = {
                "requires-network": "",
                "no-sandbox": "",
                "no-remote": "",
            },
            progress_message = "Downloading %s" % prebuilt,
        )
    return providers

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
        pkg_config = attr.string_list(
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
        prebuilt = attr.label(providers = [BuildSettingInfo]),
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
