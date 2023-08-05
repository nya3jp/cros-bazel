# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:git.bzl", "new_git_repository")

def depot_tools_repository():
    new_git_repository(
        name = "depot_tools",
        remote = "https://chromium.googlesource.com/chromium/tools/depot_tools.git",
        commit = "b7c550a6bc8be23add49a01abf64feb099c2a232",
        shallow_since = "1691174216 +0000",
        patch_cmds = [
            "touch .disable_auto_update",
            # We need gclient to fetch the chromium sources
            '''
            cat <<-'EOF' > gclient.wrapper.sh
#!/bin/bash
ROOT="$(realpath $(dirname "${BASH_SOURCE[0]}"))"
export CIPD_CACHE_DIR="${ROOT}/.cipd_cache"
export VPYTHON_VIRTUALENV_ROOT="${ROOT}/.vpython-root"
export PATH="${ROOT}:$PATH"
exec "${ROOT}/gclient" "$@"
EOF
            ''',
            "chmod +x gclient.wrapper.sh",
            # Force the cipd binaries and python venv to be downloaded.
            "DEPOT_TOOLS_DIR=$PWD ./ensure_bootstrap",
        ],
        build_file = "@//bazel/module_extensions/cros_deps:depot_tools/BUILD.depot_tools-template",
    )
