# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/bash:defs.bzl", "sh_runfiles_binary")

visibility("public")

def _pack_binary_impl(ctx):
    tarball = ctx.actions.declare_file(ctx.label.name + ".tar.gz")
    args = ctx.actions.args()
    args.add_all([tarball, ctx.attr])
    ctx.actions.run(
        executable = ctx.executable.generate_manifest,
        outputs = [tarball],
        inputs = [],
        tools = [],
        arguments = [args],
    )

    return [
        DefaultInfo(
            files = depset([tarball]),
        ),
    ]

_pack_binary = rule(
    implementation = _pack_binary_impl,
    attrs = dict(
        path = attr.string(mandatory = True),
        generate_manifest = attr.label(cfg = "exec", executable = True, mandatory = True),
    ),
)

def pack_binary(name, binary, path, **kwargs):
    manifest_generator_name = "_%s_manifest_generator" % name
    sh_runfiles_binary(
        name = manifest_generator_name,
        src = "//bazel/build_defs/packed_binary:pack_binary.sh",
        data = [binary],
    )

    tarball_name = "_%s_tarball" % name
    _pack_binary(
        name = name,
        generate_manifest = manifest_generator_name,
        path = path,
        **kwargs
    )

def _unpack_binary(ctx):
    main = ctx.actions.declare_file(ctx.label.name)
    runfiles = ctx.actions.declare_directory(ctx.label.name + "_runfiles")

    args = ctx.actions.args()
    args.add_all([ctx.file.src, main])
    ctx.actions.run(
        executable = ctx.executable._unpack_prebuilt_binary,
        inputs = [ctx.file.src],
        outputs = [main, runfiles],
        arguments = [args],
    )

    return DefaultInfo(
        files = depset([main, runfiles]),
        executable = main,
    )

unpack_binary = rule(
    implementation = _unpack_binary,
    attrs = dict(
        src = attr.label(allow_single_file = [".tar.gz"]),
        _unpack_prebuilt_binary = attr.label(cfg = "exec", executable = True, default = "//bazel/build_defs/packed_binary:unpack_binary"),
    ),
    executable = True,
)
