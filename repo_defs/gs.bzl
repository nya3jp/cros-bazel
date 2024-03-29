# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
Repository rule for downloading files from Google Cloud Storage.
"""

_BUILD_HEADER = """# AUTO-GENERATED FILE. DO NOT EDIT.
#
# File downloaded from Google Cloud Storage.
"""

_NON_EXECUTABLE_TEMPLATE = _BUILD_HEADER + """
filegroup(
    name = "file",
    srcs = ["{file}"],
    # Use public visibility since bzlmod repo namespacing prevents unwanted
    # visibility.
    visibility = ["//visibility:public"],
)
"""

_EXECUTABLE_TEMPLATE = _BUILD_HEADER + """
load("@bazel_skylib//rules:native_binary.bzl", "native_binary")

native_binary(
    name = "file",
    src = "{file}",
    out = "{file}_symlink",
    # Use public visibility since bzlmod repo namespacing prevents unwanted
    # visibility.
    visibility = ["//visibility:public"],
)
"""

GS_ATTRS = {
    "downloaded_file_path": attr.string(
        doc = "Path assigned to the downloaded file.",
    ),
    "executable": attr.bool(
        doc = "Whether the downloaded file is an executable",
        default = False,
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

    extra_env = dict()
    internal_boto = repository_ctx.workspace_root.get_child("private-overlays/chromeos-overlay/googlestorage_account.boto")
    if internal_boto.exists:
        extra_env["BOTO_CONFIG"] = str(internal_boto)

    dest = repository_ctx.path("file/" + filename)
    cmd = [
        repository_ctx.attr._gsutil,
        "cp",
        url,
        dest,
    ]
    st = repository_ctx.execute(
        cmd,
        working_directory = str(repository_ctx.workspace_root),
        environment = extra_env,
    )
    if st.return_code:
        fail("Error running command %s:\n%s%s" % (cmd, st.stdout, st.stderr))

    template = _NON_EXECUTABLE_TEMPLATE
    if repository_ctx.attr.executable:
        template = _EXECUTABLE_TEMPLATE
        st = repository_ctx.execute(["chmod", "a+x", dest])
        if st.return_code:
            fail(
                "Failed to add executable permissions to %s\n%s" %
                (dest, st.stderr),
            )

    repository_ctx.file(
        "file/BUILD.bazel",
        template.format(file = filename),
    )

gs_file = repository_rule(
    implementation = download_gs_file,
    doc = """
    Downloads a file from Google Cloud Storage and and makes it available as a
    file group.
    """,
    attrs = GS_ATTRS,
)
