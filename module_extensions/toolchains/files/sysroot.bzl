# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

visibility("public")

SysrootInfo = provider(fields = {
    "file_map": "dict[str, File]",
})

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
