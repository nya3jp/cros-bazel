# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_skylib//lib:paths.bzl", "paths")

_BUILD_TEMPLATE = """
filegroup(
    name = "src",
    srcs = ["{file}"],
    # Use public visibility since bzlmod repo namespacing prevents unwanted
    # visibility.
    visibility = ["//visibility:public"],
)
"""

def _exec(ctx, cmd, *kwargs):
    st = ctx.execute(cmd, *kwargs)
    if st.return_code != 0:
        cmd_str = " ".join(["'%s'" % (arg) for arg in cmd])
        fail("`%s`: %s" % (cmd_str, st.stderr))
    return st.stdout

def _repo_repository_impl(ctx):
    """Repository rule that clones a repo project."""

    if not ctx.which("tar"):
        fail("tar was not found on the path")

    if not ctx.which("zstd"):
        fail("zstd was not found on the path")

    if not ctx.which("git"):
        fail("git was not found on the path")

    repo_project = paths.join(str(ctx.workspace_root.dirname), ".repo", "project-objects", "%s.git" % (ctx.attr.project))

    object_type = _exec(ctx, [
        "git",
        "-C",
        repo_project,
        "cat-file",
        "-t",
        ctx.attr.tree,
    ]).strip()

    if object_type in ["tree", "commit"]:
        if ctx.attr.subdirs:
            subdirs = ctx.attr.subdirs
        else:
            subdirs = ["."]

        local_repo = "local.git"

        # Unfortunately we can't call git --worktree <dir> checkout in parallel
        # because we will get index.lock conflicts.
        # See https://lore.kernel.org/git/Y%2FUAaC4oBPIby4kV@google.com/T/#u
        # To avoid the issue we create a shallow + bare clone of the original
        # repo so the index.lock won't conflict when we run in parallel. The
        # shallow clone is just a few symlinks, so it's not expensive to create.
        _exec(
            ctx,
            [
                "git",
                "clone",
                "--shared",
                "--bare",
                repo_project,
                local_repo,
            ],
        )

        # It would be great to use git archive for creating the tarball, but
        # unfortunately the output is not hermetic.
        # See https://lore.kernel.org/git/Y%2FEFfxe0GGqnipvL@tapette.crustytoothpaste.net/T/#mb475be2c60d4faac374ea53597548a2a21a36a12
        # for a feature request to add --mtime.
        _exec(ctx, ["mkdir", "work"])
        _exec(ctx, [
            "git",
            "-C",
            local_repo,
            "--work-tree",
            ctx.path("work"),
            "checkout",
            ctx.attr.tree,
            "--",
        ] + subdirs)

        ctx.delete(local_repo)

        # See https://reproducible-builds.org/docs/archives/
        tar_common = [
            "tar",
            "--format",
            "pax",
            "--pax-option",
            "exthdr.name=%d/PaxHeaders/%f,delete=atime,delete=ctime",
            "--sort",
            "name",
            "--mtime",
            "1970-1-1 00:00Z",
            "--owner",
            "0",
            "--group",
            "0",
            "--numeric-owner",
            "--auto-compress",
            "--remove-files",
        ]
        dest_file = "%s.tar.zst" % (ctx.attr.tree)
        _exec(ctx, tar_common + [
            "--create",
            "--file",
            dest_file,
            "-C",
            "work",
            ".",
        ])
    elif object_type == "blob":
        if ctx.attr.subdirs:
            fail("subdirs can't be used with a blob")

        content = _exec(ctx, [
            "git",
            "-C",
            repo_project,
            "cat-file",
            object_type,
            ctx.attr.tree,
        ])

        dest_file = ctx.attr.tree
        ctx.file(dest_file, content)
    else:
        fail("Unknown object type '%s'" % (object_type))

    ctx.file("BUILD.bazel", _BUILD_TEMPLATE.format(file = dest_file))

# Generates a tarball from the repo project at the specified commit.
repo_repository = repository_rule(
    implementation = _repo_repository_impl,
    attrs = {
        "tree": attr.string(
            doc = """The SHA-256 of the blob, tree or commit to check out.
It's preferable to use a tree instead of a commit because multiple commits can
point to the same tree, so it buys us a little bit of de-duplication.

If the SHA is a tree or commit, then a .tar file will be produced, if the SHA
is a blob, than the raw file is extracted.

You can find the tree hash using `git ls-tree <commit> <path>`.""",
            mandatory = True,
        ),
        "project": attr.string(
            doc = """The repo project to check out.""",
            mandatory = True,
        ),
        "subdirs": attr.string_list(
            doc = """The sub-directories to clone.""",
        ),
    },
    # This is expensive to compute so we don't want to recreate it ever, if
    # possible.
    local = False,
)
