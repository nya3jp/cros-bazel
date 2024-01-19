# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
Repository rule for downloading files from CIPD.
"""

_BUILD_TEMPLATE = """
# AUTO-GENERATED FILE. DO NOT EDIT.
#
# File downloaded from CIPD.

filegroup(
    name = "file",
    srcs = ["{file}"],
    # Use public visibility since bzlmod repo namespacing prevents unwanted
    # visibility.
    visibility = ["//visibility:public"],
)
"""

def _cipd_file_impl(repository_ctx):
    protocol, path = repository_ctx.attr.url.split("://")
    if protocol != "cipd":
        fail("Expected cipd:// URL, got %s" % (repository_ctx.attr.url))

    package, version = path.split(":")

    repository_ctx.report_progress("Downloading from CIPD.")

    st = repository_ctx.execute(["mkdir", "file"])
    if st.return_code:
        fail("Error creating file dir:\n%s%s" % (st.stdout, st.stderr))

    cmd = [
        repository_ctx.workspace_root.get_child("chromium/depot_tools/cipd"),
        "pkg-fetch",
        package,
        "-version",
        version,
        "-out",
        "file/" + repository_ctx.attr.downloaded_file_path,
    ]
    st = repository_ctx.execute(cmd)
    if st.return_code:
        fail("Error running command %s:\n%s%s" % (cmd, st.stdout, st.stderr))

    repository_ctx.file(
        "file/BUILD",
        _BUILD_TEMPLATE.format(file = repository_ctx.attr.downloaded_file_path),
    )

cipd_file = repository_rule(
    implementation = _cipd_file_impl,
    doc = "Downloads a file from CIPD and and makes it available as a file group.",
    attrs = {
        "downloaded_file_path": attr.string(
            doc = """Path assigned to the downloaded file.""",
            mandatory = True,
        ),
        "url": attr.string(
            doc = """Url from where the file is downloaded.

It must start with cipd://, contain file path and version,
For example, cipd://some/tool/linux-amd64:abc1234""",
            mandatory = True,
        ),
    },
)
