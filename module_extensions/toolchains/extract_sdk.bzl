# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def _extract_sdk_impl(repo_ctx):
    out = repo_ctx.path("")

    generate_sysroot = repo_ctx.path(repo_ctx.attr._generate_sysroot)
    tarball = repo_ctx.path(repo_ctx.attr.tarball)

    args = [
        generate_sysroot,
        out,
        tarball,
    ]

    st = repo_ctx.execute(args)
    if st.return_code:
        cmdline = " ".join([str(arg) for arg in args])
        fail("Error running command '%s':\n%s%s" % (cmdline, st.stdout, st.stderr))

    repo_ctx.symlink(repo_ctx.path(repo_ctx.attr._build_file), repo_ctx.path("BUILD.bazel"))

extract_sdk = repository_rule(
    implementation = _extract_sdk_impl,
    attrs = dict(
        tarball = attr.label(allow_single_file = True, mandatory = True),
        _build_file = attr.label(default = "//bazel/module_extensions/toolchains:BUILD.sdk.bazel", allow_single_file = True),
        _generate_sysroot = attr.label(default = "//bazel/module_extensions/toolchains:generate_sysroot.py"),
    ),
)
