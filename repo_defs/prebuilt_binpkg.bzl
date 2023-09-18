# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load(":gs.bzl", "GS_ATTRS", "download_gs_file")

_BUILD_TEMPLATE = """
# AUTO-GENERATED FILE. DO NOT EDIT.
#
# File downloaded from Google Cloud Storage.

load("@@//bazel/portage/build_defs:binary_package.bzl", "binary_package")

binary_package(
    name = "binpkg",
    src = "//file",
    category = {category},
    package_name = {package_name},
    runtime_deps = {runtime_deps},
    version = {version},
    # Use public visibility since bzlmod repo namespacing prevents unwanted
    # visibility.
    visibility = ["//visibility:public"],
)
"""

def _prebuilt_binpkg_impl(repo_ctx):
    if not repo_ctx.attr.url.startswith("gs://chromeos-prebuilt/"):
        fail("Prebuilt binpkgs must come from gs://chromeos-prebuilt/")
    if not repo_ctx.attr.url.endswith(".tbz2"):
        fail("File must be a tbz2 file")

    components = repo_ctx.attr.url.split("/")
    basename = components[-1][:-5]
    category = components[-2]
    package_name, version = basename.split("-", 1)

    repo_ctx.file("BUILD.bazel", _BUILD_TEMPLATE.format(
        category = repr(category),
        package_name = repr(package_name),
        runtime_deps = repr(
            [str(label) for label in repo_ctx.attr.runtime_deps],
        ),
        version = repr(version),
    ))

    download_gs_file(repo_ctx)

prebuilt_binpkg = repository_rule(
    implementation = _prebuilt_binpkg_impl,
    doc = """
    Downloads a file from google cloud storage and makes it available as a
    prebuilt binary package.
    """,
    attrs = GS_ATTRS | {
        "runtime_deps": attr.label_list(),
    },
)
