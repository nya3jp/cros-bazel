# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

_SYMLINK_DOCS = """Symlink is similar to alias, but additionally generates a symlink.
This makes it compatible with bzlmod hub+spoke repos (since spoke repos don't
appear in the repo mapping, you can't reference them with an alias)."""

_SYMLINK_ATTRS = dict(
    actual = attr.label(allow_single_file = True, mandatory = True),
    out = attr.string(),
)

def _symlink_impl(ctx, include_target = True):
    out = ctx.actions.declare_file(ctx.attr.out or ctx.label.name)
    ctx.actions.symlink(output = out, target_file = ctx.file.actual)

    return [DefaultInfo(
        files = depset([ctx.file.actual, out] if include_target else [out]),
    )]

symlink = rule(
    doc = _SYMLINK_DOCS,
    implementation = _symlink_impl,
    attrs = _SYMLINK_ATTRS,
)

def _symlink_without_target_impl(ctx):
    return _symlink_impl(ctx, include_target = False)

# This can be used to invoke symlink with makefile substitution (which requires
# a single file).
# Typical usage:
# rule(
#   ...
#   data = [":actual", ":symlink"],
#   flags = ["$(location :symlink)"],
# )
symlink_without_target = rule(
    doc = _SYMLINK_DOCS,
    implementation = _symlink_without_target_impl,
    attrs = _SYMLINK_ATTRS,
)
