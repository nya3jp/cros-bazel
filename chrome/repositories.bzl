# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load(":cros_chrome_repository.bzl", "cros_chrome_repository")
load("@bazel_tools//tools/build_defs/repo:git.bzl", "new_git_repository")

def cros_chrome_repositories():
    new_git_repository(
        name="depot_tools",
        remote = "https://chromium.googlesource.com/chromium/tools/depot_tools.git",
        commit = "d9db3f6fd8ec38121d5255b8fbded901e7ca16eb",
        shallow_since = "1667836312 +0000",
        patch_cmds = [
            'touch .disable_auto_update',
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
            'chmod +x gclient.wrapper.sh',
            # Running gclient will force the cipd binaries and python venv to be
            # downloaded.
            './gclient.wrapper.sh help'
        ],
        build_file = "@//bazel/chrome:BUILD.depot_tools-template",
    )

    cros_chrome_repository(
        name = "chrome",
        tag = "107.0.5257.0",
        gclient = "@depot_tools//:gclient.wrapper.sh"
    )
