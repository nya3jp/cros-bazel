# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load(
    "//bazel/repo_defs:common.bzl",
    "REPO_AUTH_ENVIRON",
    _exec = "exec",
    _exec_with_gce_context_if_needed = "exec_with_gce_context_if_needed",
)

def _depot_tools_repository_impl(ctx):
    _exec(ctx, ["git", "init", "."])
    _exec(ctx, ["git", "remote", "add", "origin", ctx.attr.remote])
    _exec_with_gce_context_if_needed(
        ctx,
        ["git", "fetch", "--depth=1", "origin", ctx.attr.commit],
        msg = "Fetching depot_tools " + ctx.attr.commit,
        retries = 3,
        delay = 10,
        GIT_CONFIG_GLOBAL = "/dev/null",
    )
    _exec(ctx, ["git", "reset", "--hard", ctx.attr.commit])
    _exec_with_gce_context_if_needed(
        ctx,
        [str(ctx.path("ensure_bootstrap"))],
        msg = "Running ensure_bootstrap for depot_tools",
        retries = 3,
        delay = 10,
        DEPOT_TOOLS_UPDATE = "0",
    )
    ctx.template("BUILD.bazel", ctx.attr.build_file)

_depot_tools_repository = repository_rule(
    implementation = _depot_tools_repository_impl,
    attrs = {
        "build_file": attr.label(
            mandatory = True,
        ),
        "commit": attr.string(
            mandatory = True,
        ),
        "remote": attr.string(
            default = "https://chromium.googlesource.com/chromium/tools/depot_tools.git",
        ),
    },
    environ = REPO_AUTH_ENVIRON,
)

def depot_tools_repository():
    # TODO(b/322317375): Stop fetching the git repository and use the one at src/chromium/depot_tools.
    _depot_tools_repository(
        name = "depot_tools",
        remote = "https://chromium.googlesource.com/chromium/tools/depot_tools.git",
        commit = "86752e9a55281200715749d75a88cf57bf2e7b01",
        build_file = "@//bazel/module_extensions/cros_deps:depot_tools/BUILD.depot_tools-template",
    )
