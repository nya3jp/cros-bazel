# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/build_defs/filter_files:glob.bzl", "glob_matches")

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
    include = [tuple(glob.split("/")) for glob in ctx.attr.include]
    exclude = [tuple(glob.split("/")) for glob in ctx.attr.exclude]

    matched = {k: False for k in include}
    filtered = []

    for src in ctx.attr.srcs:
        for f in src[DefaultInfo].files.to_list():
            path = f.short_path
            if not path.startswith(strip_prefix):
                fail(_BAD_STRIP_FMT.format(
                    path = path,
                    strip_prefix = strip_prefix,
                ))
            path = path[len(strip_prefix):].lstrip("/").split("/")

            for glob in include:
                if glob_matches(path, glob):
                    if not any([glob_matches(path, g) for g in exclude]):
                        filtered.append(f)
                        matched[glob] = True
                        break

    missing_matches = [k for k, v in matched.items() if not v]
    if missing_matches and not ctx.attr.allow_empty:
        fail(_NO_MATCH_FMT.format(
            globs = ["/".join(glob) for glob in missing_matches],
            f = "/".join(path),
        ))

    return DefaultInfo(files = depset(filtered))

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
