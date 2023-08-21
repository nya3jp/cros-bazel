# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@rules_pkg//pkg:tar.bzl", "pkg_tar")
load("//bazel/portage/build_defs:common.bzl", "BinaryPackageInfo", "BinaryPackageSetInfo", "single_binary_package_set_info")
load("//bazel/portage/build_defs:binary_package.bzl", "binary_package")

def direct_ebuild(
        name,
        package,
        category,
        package_name,
        version,
        slot,
        runtime_deps = [],
        visibility = None):
    # Make package an absolute label.
    if ":" not in package:
        package += ":" + package.rsplit("/", 1)[-1]
    if not package.startswith("@"):
        package = "@@" + package

    tar_name = "_%s_tbz2" % name
    pkg_tar(
        name = tar_name,
        out = name + ".tbz2",
        srcs = [package],
        compressor = "@@//bazel/cros_pkg/private:gen_tbz2",
        compressor_args = " ".join([
            "--category=" + category,
            "--package-name=" + package_name,
            "--version=" + version,
            "--slot=" + slot,
        ]),
        visibility = ["//visibility:private"],
    )

    binary_package(
        name = name,
        category = category,
        package_name = package_name,
        version = version,
        src = tar_name,
        runtime_deps = runtime_deps,
        visibility = visibility,
    )
