# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

_NOW_REPO_BUILD_FILE = """
# Depend on this file if you want a target to always rerun.
exports_files(["date"])
"""

def _now_repository_impl(repo_ctx):
    """Repository rule to generate a file that always changes."""

    repo_ctx.file("BUILD.bazel", content = _NOW_REPO_BUILD_FILE)

    date = repo_ctx.os.environ.get("CACHE_BUST_DATE")
    repo_ctx.file("date", content = date)

now = repository_rule(
    implementation = _now_repository_impl,
    environ = [
        # See tools/bazel for where this variable is set
        "CACHE_BUST_DATE",
    ],
    local = True,
)
