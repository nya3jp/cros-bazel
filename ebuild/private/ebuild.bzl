# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/ebuild/private/common/mountsdk:mountsdk.bzl", "COMMON_ATTRS", "debuggable_mountsdk", "mountsdk_generic")
load("@bazel_skylib//lib:paths.bzl", "paths")

def _ebuild_basename(ctx):
    return ctx.file.ebuild.basename.rsplit(".", 1)[0]

def _ebuild_declare_outputs(ctx, args, file_type, allowed_extensions = None):
    basename = _ebuild_basename(ctx)

    outputs = []
    for inside_path in getattr(ctx.attr, file_type):
        if not paths.is_absolute(inside_path):
            fail("%s is not absolute" % inside_path)
        file_name = paths.basename(inside_path)

        _, extension = paths.split_extension(file_name)
        if allowed_extensions and extension not in allowed_extensions:
            fail("%s must be of file type %s, got %s" % (inside_path, allowed_extensions, extension))

        file = ctx.actions.declare_file(paths.join(basename, "files", file_type, file_name))
        outputs.append(file)
        args.add("--output-file=%s=%s" % (inside_path, file.path))

    return outputs

def _ebuild_impl(ctx):
    src_basename = _ebuild_basename(ctx)
    binpkg_output_file = ctx.actions.declare_file(src_basename + ".tbz2")

    args = ctx.actions.args()
    args.add_all([
        "--ebuild=" + ctx.file.ebuild.path,
    ])

    xpak_outputs = []
    for xpak_file in ["NEEDED", "REQUIRES", "PROVIDES"]:
        file = ctx.actions.declare_file(paths.join(src_basename, "xpak", xpak_file))
        xpak_outputs.append(file)
        args.add("--xpak=%s=?%s" % (xpak_file, file.path))

    header_outputs = _ebuild_declare_outputs(
        ctx,
        args,
        "headers",
        allowed_extensions = [".h", ".hpp"],
    )
    pkg_config_outputs = _ebuild_declare_outputs(
        ctx,
        args,
        "pkg_config",
        allowed_extensions = [".pc"],
    )
    shared_lib_outputs = _ebuild_declare_outputs(ctx, args, "shared_libs")
    static_lib_outputs = _ebuild_declare_outputs(
        ctx,
        args,
        "static_libs",
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

    outputs = ([binpkg_output_file] +
               xpak_outputs +
               header_outputs +
               pkg_config_outputs +
               shared_lib_outputs +
               static_lib_outputs)

    return mountsdk_generic(
        ctx,
        progress_message_name = ctx.file.ebuild.basename,
        inputs = [ctx.file.ebuild],
        binpkg_output_file = binpkg_output_file,
        outputs = outputs,
        args = args,
        extra_providers = [output_group_info],
        install_deps = True,
    )

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
        _builder = attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/build_package"),
        ),
        **COMMON_ATTRS
    ),
)

def ebuild(name, **kwargs):
    debuggable_mountsdk(name = name, orig_rule = _ebuild, **kwargs)
