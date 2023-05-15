# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

_SYSROOT_CONTENTS = """
# Avoid using this wherever possible, as it will be incompatible with RBE.
ABSOLUTE_SYSROOT_PATH = "%s"
"""

_TAR_ARGS = [
    [
        ".",
        "--exclude=./usr/x86_64-cros-linux-gnu",
        "--exclude=./usr/x86_64-pc-linux-gnu",
    ],
    [
        "./usr/x86_64-cros-linux-gnu",
        "--strip-components=3",
        # These are hardlinks and don't really work when we extract seperately.
        "--exclude=./usr/x86_64-cros-linux-gnu/usr/lib64/gconv",
    ],
    [
        "./usr/x86_64-pc-linux-gnu",
        "--strip-components=3",
    ],
]

def _extract_sdk_impl(repo_ctx):
    out = repo_ctx.path("")
    repo_ctx.file(
        out.get_child("sysroot.bzl"),
        content = _SYSROOT_CONTENTS % str(out),
    )

    # This leaves significant performance on the table and should be pretty easy
    # to optimize later on.
    # Just running tar --list <archive> seems to take ~1 minute, so creating an
    # executable to untar should at least shave 2 minutes off the execution.
    # More if we can parallelize the extraction.
    tar = repo_ctx.which("tar")
    if not tar:
        fail("tar was not found on the path")

    base_args = [
        tar,
        "-xf",
        repo_ctx.path(repo_ctx.attr.tarball),
        "-C",
        out,
    ]

    for args in _TAR_ARGS:
        args = base_args + args
        st = repo_ctx.execute(args)
        if st.return_code:
            cmdline = " ".join([str(arg) for arg in args])
            fail("Error running command '%s':\n%s%s" % (cmdline, st.stdout, st.stderr))

    if repo_ctx.attr._build_file:
        repo_ctx.symlink(repo_ctx.path(repo_ctx.attr._build_file), repo_ctx.path("BUILD.bazel"))

extract_sdk = repository_rule(
    implementation = _extract_sdk_impl,
    attrs = dict(
        tarball = attr.label(allow_single_file = True, mandatory = True),
        _build_file = attr.label(default = "//bazel/module_extensions/toolchains:BUILD.sdk.bazel", allow_single_file = True),
    ),
)
