# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

_SYMLINK_DOCS = """Symlink is similar to alias, but additionally generates a symlink.
This makes it compatible with bzlmod hub+spoke repos (since spoke repos don't
appear in the repo mapping, you can't reference them with an alias)."""

def _symlink_impl(ctx):
    files = ctx.files.actual
    if len(files) != 1:
        fail("Symlink must be called on a single file")
    file = files[0]
    out = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.symlink(output = out, target_file = file)

    return [DefaultInfo(
        files = depset([file, out]),
    )]

symlink = rule(
    doc = _SYMLINK_DOCS,
    implementation = _symlink_impl,
    attrs = dict(
        actual = attr.label(allow_single_file = True, mandatory = True),
    ),
)
