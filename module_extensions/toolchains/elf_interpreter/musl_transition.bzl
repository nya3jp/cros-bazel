# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

visibility("private")

def _musl_transition_impl(settings, attr):
    _ignore = (settings, attr)
    return {
        "//command_line_option:platforms": [Label("//bazel/platforms:host_musl")],
    }

_musl_transition = transition(
    implementation = _musl_transition_impl,
    inputs = [],
    outputs = ["//command_line_option:platforms"],
)

def _musl_transition_rule_impl(ctx):
    return DefaultInfo(
        files = depset([ctx.executable.actual]),
    )

musl_transition = rule(
    implementation = _musl_transition_rule_impl,
    attrs = dict(
        actual = attr.label(
            executable = True,
            cfg = _musl_transition,
        ),
        _allowlist_function_transition = attr.label(
            default = "@bazel_tools//tools/allowlists/function_transition_allowlist",
        ),
    ),
)
