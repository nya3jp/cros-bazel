# Copyright 2026 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load(
    "//bazel/repo_defs:common.bzl",
    "REPO_AUTH_ENVIRON",
    _exec = "exec",
)

def _git_wrapper_repository_impl(ctx):
    cipd_bin = ctx.workspace_root.get_child("chromium/depot_tools/cipd")
    ctx.file("cipd.ensure", "infra/tools/git/${platform} %s\n" % ctx.attr.version)
    _exec(
        ctx,
        [
            cipd_bin,
            "ensure",
            "-root",
            ".",
            "-ensure-file",
            "cipd.ensure",
        ],
        msg = "Downloading Git wrapper from CIPD",
    )
    ctx.file(
        "BUILD.bazel",
        """
exports_files(["git"])
""",
    )

_git_wrapper_repository = repository_rule(
    implementation = _git_wrapper_repository_impl,
    attrs = {
        "version": attr.string(
            default = "latest",
            doc = "CIPD package version or instance ID for infra/tools/git/${platform}",
        ),
    },
    environ = REPO_AUTH_ENVIRON,
)

def git_wrapper_repository(version = "latest"):
    _git_wrapper_repository(
        name = "git_wrapper",
        version = version,
    )
