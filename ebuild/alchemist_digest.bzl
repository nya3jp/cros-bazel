# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

_ALCHEMIST_DIGEST_REPO_BUILD_FILE = """
exports_files(["digest", "board"])
"""

def _alchemist_digest_repository_impl(ctx):
    """Repository rule to generate a digest of the boards overlays."""

    # Keep all the ctx.path calls first to avoid expensive restarts
    alchemist = ctx.path(ctx.attr._alchemist)

    # --source-dir needs the repo root, not just the `src` directory
    root = ctx.workspace_root.dirname

    # BOARD has the format <board>:<profile>
    board = ctx.os.environ.get("BOARD", "")
    parts = board.split(":", 1)
    if len(parts) > 1:
        board = parts[0]
        profile = parts[1]
    else:
        profile = ""

    # If we don't have a BOARD defined, we need to clear out the repository
    if board:
        # TODO: add a cache_dir argument
        args = [
            alchemist,
            "--board",
            board,
            "--source-dir",
            root,
            "digest-repo",
        ]
        st = ctx.execute(args)
        if st.return_code:
            fail("Error running command %s:\n%s%s" % (args, st.stdout, st.stderr))

        digest = st.stdout
    else:
        digest = ""

    ctx.file("BUILD.bazel", content = _ALCHEMIST_DIGEST_REPO_BUILD_FILE)

    # Pass the config to the @portage repo
    ctx.file("board", content = board)
    ctx.file("profile", content = profile)
    ctx.file("digest", content = digest)

alchemist_digest = repository_rule(
    implementation = _alchemist_digest_repository_impl,
    environ = [
        # See tools/bazel for where this variable is set
        "_CACHE_BUST_DATE",
        "BOARD",
    ],
    attrs = {
        "_alchemist": attr.label(
            default = Label("@alchemist//:release/alchemist"),
            allow_single_file = True,
        ),
    },
    local = True,
)
