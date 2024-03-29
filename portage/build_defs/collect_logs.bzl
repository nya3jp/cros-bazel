# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/portage/build_defs:common.bzl", "TransitiveLogsInfo", "get_all_deps")

def _collect_logs_aspect_impl(target, ctx):
    depsets = []

    output_groups = target[OutputGroupInfo]
    for name in ("logs", "traces"):
        d = getattr(output_groups, name, None)
        if d:
            depsets.append(d)

    for dep in get_all_deps(ctx):
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
