# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def _cros_sdk_repository_impl(ctx):
    """Implementation of the http_archive rule that supports add_prefix."""

    file = "{}.{}".format(ctx.name, ctx.attr.type)
    url = "https://commondatastorage.googleapis.com/chromiumos-sdk/{}".format(file)

    ctx.report_progress("Downloading {}".format(file))
    download_info = ctx.download(
        url,
        output = file,
        sha256 = ctx.attr.sha256,
    )

    # If the user has pixz installed use that to extract, otherwise fall back to tar.
    # We can't use ctx.download_and_extract because it mangles the symlinks.
    cmds = []

    mkdir = ctx.which("mkdir")
    if not mkdir:
        fail("mkdir not found")
    cmds.append([mkdir, "root"])

    tar = ctx.which("tar")
    if not tar:
        fail("tar was not found on the path")

    pixz = ctx.which("pixz")
    if pixz:
        ctx.report_progress("Extracting {} with pixz".format(file))
        cmds.append([tar, "-I{}".format(pixz), "-xf", file, "-C", "root"])
    else:
        ctx.report_progress("Extracting {} with tar".format(file))
        cmds.append([tar, "-xf", file, "-C", "root"])

    for cmd in cmds:
        st = ctx.execute(cmd)
        if st.return_code:
            fail("Error running patch command %s:\n%s%s" %
                 (cmd, st.stdout, st.stderr))

    ctx.file("WORKSPACE", "workspace(name = \"{name}\")\n".format(name = ctx.name))
    ctx.file("BUILD.bazel", "exports_files([\"root\"])")

    ctx.delete(file)

_cros_sdk_repository_attrs = {
    "sha256": attr.string(
        doc = """The expected SHA-256 of the file downloaded.""",
        mandatory = True,
    ),
    "type": attr.string(
        doc = """The archive type of the downloaded file.""",
        default = "tar.xz",
    ),
}

cros_sdk_repository = repository_rule(
    implementation = _cros_sdk_repository_impl,
    attrs = _cros_sdk_repository_attrs,
)
