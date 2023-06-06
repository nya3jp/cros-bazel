# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def _chromite_impl(repo_ctx):
    # While most repo rules would inject BUILD.project-chromite during the repo
    # rule, since we perform a symlink, doing so would add it to the real
    # chromite directory.
    realpath = str(repo_ctx.workspace_root.realpath).rsplit("/", 1)[0]
    out = repo_ctx.path(".")
    repo_ctx.symlink(realpath + "/chromite", out)

chromite = repository_rule(
    implementation = _chromite_impl,
)
