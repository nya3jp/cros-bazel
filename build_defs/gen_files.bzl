# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

visibility("public")

def _gen_files_impl(ctx):
    outs = []
    for name, content in ctx.attr.file_contents.items():
        out = ctx.actions.declare_file(name)
        ctx.actions.write(out, content)
        outs.append(out)
    return DefaultInfo(files = depset(outs))

gen_files = rule(
    implementation = _gen_files_impl,
    attrs = dict(
        file_contents = attr.string_dict(mandatory = True),
    ),
)
