# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def _new_local_repository_impl(repo_ctx):
    path = repo_ctx.workspace_root.get_child(repo_ctx.attr.path)

    entries = repo_ctx.path(path).readdir()
    for entry in entries:
        link_name = "BUILD.bazel.orig" if entry.basename == "BUILD.bazel" else entry.basename
        repo_ctx.symlink(entry, link_name)
    repo_ctx.symlink(repo_ctx.attr.build_file, "BUILD.bazel")

new_local_repository = repository_rule(
    implementation = _new_local_repository_impl,
    attrs = dict(
        build_file = attr.label(mandatory = True, allow_single_file = True),
        path = attr.string(mandatory = True),
    ),
)
