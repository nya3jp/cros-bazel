# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

_EMPTY_PORTAGE_BUILD = """
package_group(
    name = "all_packages",
    packages = [
        "//...",
    ],
)
"""

def _portage_impl(repo_ctx):
    """Repository rule to generate the board's bazel BUILD files."""

    # Keep all the repo_ctx.path calls first to avoid expensive restarts
    alchemist = repo_ctx.path(repo_ctx.attr.alchemist)
    board = repo_ctx.read(repo_ctx.attr.board)
    profile = repo_ctx.read(repo_ctx.attr.profile)

    # Ensure the repo rule reruns when the digest changes.
    repo_ctx.path(repo_ctx.attr.digest)

    out = repo_ctx.path("")

    # --source-dir needs the repo root, not just the `src` directory
    root = repo_ctx.workspace_root.dirname

    # If we don't have a BOARD defined, we need to clear out the repository
    if board:
        args = [
            alchemist,
            "--board",
            board,
            "--source-dir",
            root,
        ]
        if profile:
            args += ["--profile", profile]

        args += [
            "generate-repo",
            "--output-dir",
            out,
            "--output-repos-json",
            repo_ctx.path("deps.json"),
        ]

        st = repo_ctx.execute(args, quiet = False)
        if st.return_code:
            fail("Error running command %s" % (args,))

    else:
        # TODO: Consider running alchemist in this case as well. Then we don't
        # need this special logic.
        repo_ctx.file("deps.json", content = "{}")
        repo_ctx.file(out.get_child("settings.bzl"), content = "BOARD = None")
        repo_ctx.file(out.get_child("BUILD.bazel"), content = _EMPTY_PORTAGE_BUILD)

portage = repository_rule(
    implementation = _portage_impl,
    attrs = dict(
        board = attr.label(allow_single_file = True),
        profile = attr.label(allow_single_file = True),
        digest = attr.label(
            doc = "Used to trigger this rule to rerun when the overlay contents change",
            allow_single_file = True,
        ),
        alchemist = attr.label(allow_single_file = True),
    ),
    # Do not set this to true. It will force the evaluation to happen every
    # bazel invocation for unknown reasons...
    local = False,
)