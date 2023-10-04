# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@rules_pkg//pkg:tar.bzl", "pkg_tar")
load("//bazel/portage/build_defs:binary_package.bzl", "binary_package")

visibility("public")

def direct_ebuild(
        name,
        package,
        category,
        package_name,
        version,
        slot = "0/0",
        runtime_deps = [],
        visibility = None):
    tar_name = "_%s_tbz2" % name
    pkg_tar(
        name = tar_name,
        out = name + ".tbz2",
        srcs = [package],
        compressor = "@@//bazel/cros_pkg/private:gen_tbz2",
        build_tar = "//bazel/cros_pkg/private:build_tar",
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
        slot = slot,
        src = tar_name,
        runtime_deps = runtime_deps,
        visibility = visibility,
    )
