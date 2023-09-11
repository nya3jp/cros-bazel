# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

visibility("private")

def _runfiles_to_files_impl(ctx):
    info = ctx.attr.actual[DefaultInfo]
    return [DefaultInfo(
        files = depset(transitive = [
            info.files,
            info.default_runfiles.files,
        ]),
    )]

runfiles_to_files = rule(
    implementation = _runfiles_to_files_impl,
    attrs = dict(
        actual = attr.label(executable = True, cfg = "exec"),
    ),
)
