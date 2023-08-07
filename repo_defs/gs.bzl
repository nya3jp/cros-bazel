# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
Repository rule for downloading files from Google Cloud Storage.
"""

_BUILD_TEMPLATE = """
# AUTO-GENERATED FILE. DO NOT EDIT.
#
# File downloaded from Google Cloud Storage.

filegroup(
    name = "file",
    srcs = ["{file}"],
    # Use public visibility since bzlmod repo namespacing prevents unwanted
    # visibility.
    visibility = ["//visibility:public"],
)
"""

def _gs_file_impl(repository_ctx):
    repository_ctx.report_progress("Downloading from GS.")
    repository_ctx.execute(["mkdir", "file"])
    repository_ctx.execute([
        repository_ctx.attr._gsutil,
        "cp",
        repository_ctx.attr.url,
        "file/" + repository_ctx.attr.downloaded_file_path,
    ])
    repository_ctx.file(
        "file/BUILD.bazel",
        _BUILD_TEMPLATE.format(file = repository_ctx.attr.downloaded_file_path),
    )

gs_file = repository_rule(
    implementation = _gs_file_impl,
    doc = """
    Downloads a file from Google Cloud Storage and and makes it available as a
    file group.
    """,
    attrs = {
        "downloaded_file_path": attr.string(
            doc = "Path assigned to the downloaded file.",
            mandatory = True,
        ),
        "url": attr.string(
            doc = "gs:// URL from where the file is downloaded.",
            mandatory = True,
        ),
        "_gsutil": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("@chromite//:bin/gsutil"),
        ),
    },
)
