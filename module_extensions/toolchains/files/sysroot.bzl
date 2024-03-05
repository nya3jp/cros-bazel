# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Rules for working with sysroots."""

load("//bazel/build_defs/filter_files:glob.bzl", "glob")

visibility("public")

SysrootInfo = provider(
    "A sysroot containing files.",
    fields = {
        "file_map": "dict[str, File]",
    },
)

def _sysroot_impl(ctx):
    strip_prefix = ctx.attr.strip_prefix.rstrip("/") + "/"

    file_map = {}
    for file in ctx.files.srcs:
        path = file.short_path
        if not path.startswith(strip_prefix):
            fail("Expected %r to start with the strip_prefix %r" % (path, strip_prefix))
        file_map["/" + path[len(strip_prefix):]] = file

    for link, target in ctx.attr.symlinks.items():
        f = ctx.actions.declare_file(link.lstrip("/"))
        ctx.actions.symlink(output = f, target_file = file_map[target])
        file_map[link] = f

    return [
        DefaultInfo(files = depset(file_map.values())),
        SysrootInfo(file_map = file_map),
    ]

sysroot = rule(
    implementation = _sysroot_impl,
    attrs = dict(
        srcs = attr.label_list(allow_files = True),
        strip_prefix = attr.string(mandatory = True),
        symlinks = attr.string_dict(),
    ),
)

def _sysroot_glob_impl(ctx):
    return [DefaultInfo(files = depset(glob(
        ctx.attr.sysroot[SysrootInfo].file_map,
        include = ctx.attr.include,
        exclude = ctx.attr.exclude,
        allow_empty = ctx.attr.allow_empty,
    ).values()))]

sysroot_glob = rule(
    implementation = _sysroot_glob_impl,
    attrs = dict(
        sysroot = attr.label(mandatory = True, providers = [SysrootInfo]),
        include = attr.string_list(mandatory = True),
        exclude = attr.string_list(default = []),
        allow_empty = attr.bool(default = False),
    ),
)

def _sysroot_single_file_impl(ctx):
    file = ctx.attr.sysroot[SysrootInfo].file_map[ctx.attr.path]
    return [DefaultInfo(files = depset([file]))]

sysroot_single_file = rule(
    implementation = _sysroot_single_file_impl,
    attrs = dict(
        sysroot = attr.label(mandatory = True, providers = [SysrootInfo]),
        path = attr.string(mandatory = True),
    ),
)
