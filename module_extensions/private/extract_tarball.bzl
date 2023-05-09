# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def _extract_tarball_impl(repo_ctx):
    tar = repo_ctx.which("tar")
    if not tar:
        fail("tar was not found on the path")
    out = repo_ctx.path("")

    args = [
        tar,
        "-xf",
        repo_ctx.path(repo_ctx.attr.tarball),
        "-C",
        out,
        ".",
    ]
    st = repo_ctx.execute(args)
    if st.return_code:
        cmdline = " ".join([str(arg) for arg in args])
        fail("Error running command '%s':\n%s%s" % (cmdline, st.stdout, st.stderr))

    if repo_ctx.attr.build_file:
        repo_ctx.symlink(repo_ctx.path(repo_ctx.attr.build_file), repo_ctx.path("BUILD.bazel"))

extract_tarball = repository_rule(
    implementation = _extract_tarball_impl,
    attrs = dict(
        tarball = attr.label(allow_single_file = True, mandatory = True),
        build_file = attr.label(allow_single_file = True),
    ),
)
