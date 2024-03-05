# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/build_defs/filter_files:glob.bzl", "glob")

visibility("public")

_NO_MATCH_FMT = """The following entries failed to match anything. If this is \
expected, consider setting allow_entry to True in the filter_files invocation.
{globs}

Example file: "{f}"
"""
_BAD_STRIP_FMT = 'Unable to strip prefix - the path "{path}" does not start \
with "{strip_prefix}"'

def _filter_files_impl(ctx):
    strip_prefix = ctx.attr.strip_prefix

    file_map = {}
    for f in ctx.files.srcs:
        path = f.short_path
        if not path.startswith(strip_prefix):
            fail(_BAD_STRIP_FMT.format(
                path = path,
                strip_prefix = strip_prefix,
            ))
        file_map[path[len(strip_prefix):].lstrip("/")] = f

    return [DefaultInfo(files = depset(glob(
        file_map,
        include = ctx.attr.include,
        exclude = ctx.attr.exclude,
        allow_empty = ctx.attr.allow_empty,
    ).values()))]

filter_files = rule(
    implementation = _filter_files_impl,
    attrs = dict(
        srcs = attr.label_list(mandatory = True),
        strip_prefix = attr.string(mandatory = True),
        include = attr.string_list(mandatory = True),
        exclude = attr.string_list(default = []),
        allow_empty = attr.bool(default = False),
    ),
)
