# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

_VPYTHON_INFO_REPO_BUILD_FILE = """
exports_files(["vpython_info"])
"""

def _vpython_info_repository_impl(repo_ctx):
    """Repository rule to generate info needed to use vpython."""

    # Use VPYTHON_VIRTUALENV_ROOT as the virtualenv root if specified.
    # Otherwise, use "$HOME/.vpython-root" if exists.
    virtualenv_root = repo_ctx.os.environ.get("VPYTHON_VIRTUALENV_ROOT")
    if not virtualenv_root:
        home = repo_ctx.os.environ.get("HOME")
        if home:
            default_virtualenv_root = home + "/.vpython-root"
            if repo_ctx.path(default_virtualenv_root).exists:
                virtualenv_root = default_virtualenv_root

    if virtualenv_root:
        print("Using vpython virtualenv root: " + virtualenv_root)

    vpython_info = json.encode({
        "virtualenv_root": virtualenv_root,
    })
    repo_ctx.file("vpython_info", content = vpython_info)
    repo_ctx.file("BUILD.bazel", content = _VPYTHON_INFO_REPO_BUILD_FILE)

vpython_info = repository_rule(
    implementation = _vpython_info_repository_impl,
    environ = [
        "HOME",
        "VPYTHON_VIRTUALENV_ROOT",
    ],
    local = True,
)
