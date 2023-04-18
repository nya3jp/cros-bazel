# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

_ALCHEMIST_REPO_REPOSITORIES_FILE = """
# AUTO-GENERATED FILE. DO NOT EDIT.

def portage_dependencies():
    pass
"""

def _alchemist_repo_repository_impl(ctx):
    """Repository rule to generate the board's bazel BUILD files."""

    # Keep all the ctx.path calls first to avoid expensive restarts
    alchemist = ctx.path(ctx.attr._alchemist)
    board = ctx.read(ctx.attr._board)
    # TODO: Alchemist doesn't like `.` as an --output-dir so pass in an absolute
    # path for now.
    out = ctx.path("")

    # --source-dir needs the repo root, not just the `src` directory
    root = ctx.workspace_root.dirname

    # If we don't have a BOARD defined, we need to clear out the repository
    if board:
        args = [
            alchemist,
            "--board",
            board,
            "--source-dir",
            root,
            "generate-repo",
            "--output-dir",
            out,
        ]
        st = ctx.execute(args, quiet=False)
        if st.return_code:
            fail("Error running command %s:\n%s%s" % (args, st.stdout, st.stderr))
    else:
        ctx.file("repositories.bzl", content = _ALCHEMIST_REPO_REPOSITORIES_FILE)
        ctx.file("BUILD.bazel", content = "")

# TODO: This rule depend on the user having ran `setup_board --board <BOARD>`
# inside the `cros_sdk`.
alchemist_repo = repository_rule(
    implementation = _alchemist_repo_repository_impl,
    attrs = {
        "_board": attr.label(
            default = Label("@portage_digest//:board"),
            allow_single_file = True,
        ),
        "_digest": attr.label(
            default = Label("@portage_digest//:digest"),
            doc = "Used to trigger this rule to rerun when the overlay contents change",
            allow_single_file = True,
        ),
        "_alchemist": attr.label(
            default = Label("@alchemist//:release/alchemist"),
            allow_single_file = True,
        ),
    },
    # Do not set this to true. It will force the evaluation to happen every
    # bazel invocation for unknown reasons...
    local = False,
)
