# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/portage/build_defs:common.bzl", "TransitiveLogsInfo")

def _get_all_deps(ctx):
    """
    Collects all dependencies specified in the attributes of the current rule.

    Args:
        ctx: ctx: Context passed to the current rule implementation function.

    Returns:
        list[Depset[Target]]: All dependency targets.
    """
    deps = []
    for name in dir(ctx.rule.attr):
        attr = getattr(ctx.rule.attr, name)
        if type(attr) == "Target":
            deps.append(attr)
        elif type(attr) == "list":
            for value in attr:
                if type(value) == "Target":
                    deps.append(value)
        elif type(attr) == "dict":
            for key, value in attr.items():
                if type(key) == "Target":
                    deps.append(key)
                if type(value) == "Target":
                    deps.append(value)
    return deps

def _collect_logs_aspect_impl(target, ctx):
    depsets = []

    output_groups = target[OutputGroupInfo]
    for name in ("logs", "traces"):
        d = getattr(output_groups, name, None)
        if d:
            depsets.append(d)

    for dep in _get_all_deps(ctx):
        if TransitiveLogsInfo in dep:
            depsets.append(dep[TransitiveLogsInfo].files)

    logs = depset(transitive = depsets)

    return [
        TransitiveLogsInfo(files = logs),
        OutputGroupInfo(transitive_logs = logs),
    ]

collect_logs_aspect = aspect(
    implementation = _collect_logs_aspect_impl,
    attr_aspects = ["*"],
    doc = """
    Collects all log files in the transitive dependencies and make them
    available as the output group named "transitive_logs".
    """,
)
