# Copyright 2023 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
Repository rule for downloading files from CIPD.
"""

_BUILD_TEMPLATE = """
# AUTO-GENERATED FILE. DO NOT EDIT.
#
# File downloaded from CIPD.

package(default_visibility = ["//visibility:public"])

filegroup(
    name = "file",
    srcs = ["{file}"],
)
"""

def _cipd_file_impl(repository_ctx):
    package, version = repository_ctx.attr.url.lstrip("cipd://").split(":")
    repository_ctx.report_progress("Downloading from CIPD.")
    repository_ctx.execute(["mkdir", "file"])
    repository_ctx.execute([
        repository_ctx.attr._cipd,
        "pkg-fetch",
        package,
        "-version",
        version,
        "-out",
        "file/" + repository_ctx.attr.downloaded_file_path
    ])
    repository_ctx.file(
        "file/BUILD",
        _BUILD_TEMPLATE.format(file = repository_ctx.attr.downloaded_file_path)
    )

cipd_file = repository_rule(
    implementation = _cipd_file_impl,
    doc = "Downloads a file from CIPD and and makes it available as a file group.",
    attrs = {
        "downloaded_file_path": attr.string(
            doc = """Path assigned to the downloaded file.""",
            mandatory = True
        ),
        "url": attr.string(
            doc = """Url from where the file is downloaded.

It must start with cipd://, contain file path and version,
For example, cipd://some/tool/linux-amd64:abc1234""",
            mandatory = True
        ),
        "_cipd": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("@depot_tools//:cipd"),
        ),
    },
)
