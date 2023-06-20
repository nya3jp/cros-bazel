# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/ebuild/private/alchemist:src.bzl", "ALCHEMIST_SRCS")

_ALCHEMIST_PATH = "bazel/ebuild/private/alchemist"
_ALCHEMIST_LOCK = "//:Cargo.lock"
_ALCHEMIST_CARGO_TOML = "//bazel/ebuild/private/alchemist:Cargo.toml"
_ALCHEMIST_PROFILE = "release"

def _alchemist_impl(repo_ctx):
    cargo = repo_ctx.path(repo_ctx.attr._cargo)
    target_dir = repo_ctx.path("target")
    args = [
        cargo,
        "build",
        "--package",
        "alchemist",
        "--target-dir",
        target_dir,
        "--profile",
        _ALCHEMIST_PROFILE,
    ]
    working_dir = repo_ctx.workspace_root.get_child(_ALCHEMIST_PATH)
    st = repo_ctx.execute(args, working_directory = str(working_dir))
    if st.return_code:
        fail("Error running command %s:\n%s%s" %
             (args, st.stdout, st.stderr))
    repo_ctx.symlink("target/%s/alchemist" % _ALCHEMIST_PROFILE, "alchemist")
    repo_ctx.file(repo_ctx.path("BUILD.bazel"), 'exports_files(["alchemist"])')

alchemist = repository_rule(
    implementation = _alchemist_impl,
    attrs = dict(
        _cargo = attr.label(default = Label("@rust_host_tools//:bin/cargo")),
        _srcs = attr.label_list(default = ALCHEMIST_SRCS),
        _cargo_lockfile = attr.label(default = _ALCHEMIST_LOCK),
        _cargo_toml = attr.label(default = _ALCHEMIST_CARGO_TOML),
    ),
)
