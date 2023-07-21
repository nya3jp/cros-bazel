# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

visibility("public")

def _always_fail_impl(ctx):
    fail(ctx.attr.message)

always_fail = rule(
    implementation = _always_fail_impl,
    doc = "Triggers Bazel analysis failure with the specified message.",
    attrs = {
        "message": attr.string(
            doc = "The error message to print.",
            mandatory = True,
        ),
    },
)
