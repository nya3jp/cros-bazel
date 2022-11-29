# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/ebuild/private/common/mountsdk:mountsdk.bzl", "COMMON_ATTRS", "debuggable_mountsdk", "mountsdk_generic")

def _ebuild_impl(ctx):
    src_basename = ctx.file.ebuild.basename.rsplit(".", 1)[0]
    output = ctx.actions.declare_file(src_basename + ".tbz2")

    args = ctx.actions.args()
    args.add_all([
        "--ebuild=" + ctx.file.ebuild.path,
    ])
    return mountsdk_generic(
        ctx,
        progress_message_name = ctx.file.ebuild.basename,
        inputs = [ctx.file.ebuild],
        output = output,
        args = args,
    )

_ebuild = rule(
    implementation = _ebuild_impl,
    attrs = dict(
        ebuild = attr.label(
            mandatory = True,
            allow_single_file = [".ebuild"],
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
