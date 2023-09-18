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

GS_ATTRS = {
    "downloaded_file_path": attr.string(
        doc = "Path assigned to the downloaded file.",
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
}

def download_gs_file(repository_ctx):
    repository_ctx.report_progress("Downloading from GS.")
    repository_ctx.execute(["mkdir", "file"])

    url = repository_ctx.attr.url
    if not url.startswith("gs://"):
        fail("URL must start with \"gs://\". Got %s" % repository_ctx.url)

    filename = repository_ctx.attr.downloaded_file_path
    if not filename:
        filename = url.split("/")[-1]

    repository_ctx.execute([
        repository_ctx.attr._gsutil,
        "cp",
        url,
        "file/" + filename,
    ])
    repository_ctx.file(
        "file/BUILD.bazel",
        _BUILD_TEMPLATE.format(file = filename),
    )

gs_file = repository_rule(
    implementation = download_gs_file,
    doc = """
    Downloads a file from Google Cloud Storage and and makes it available as a
    file group.
    """,
    attrs = GS_ATTRS,
)
