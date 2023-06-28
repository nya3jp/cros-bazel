# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@rules_pkg//pkg:tar.bzl", "pkg_tar")
load("//bazel/ebuild/private:common.bzl", "BinaryPackageInfo", "BinaryPackageSetInfo", "single_binary_package_set_info")

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

    _direct_ebuild_providers(
        name = name,
        tarball = tar_name,
        category = category,
        runtime_deps = runtime_deps,
        visibility = visibility,
    )

def _direct_ebuild_providers_impl(ctx):
    tarball = ctx.file.tarball
    runtime_deps = [dep[BinaryPackageInfo] for dep in ctx.attr.runtime_deps]

    binpkg_info = BinaryPackageInfo(
        file = tarball,
        category = ctx.attr.category,
        all_files = depset(
            [tarball],
            transitive = [dep.all_files for dep in runtime_deps],
            order = "postorder",
        ),
        direct_runtime_deps = tuple(runtime_deps),
        transitive_runtime_deps = depset(
            transitive = [
                depset(
                    [dep],
                    transitive = [dep.transitive_runtime_deps],
                    order = "postorder",
                )
                for dep in runtime_deps
            ],
            order = "postorder",
        ),
    )

    return [
        DefaultInfo(files = depset([tarball])),
        binpkg_info,
        single_binary_package_set_info(binpkg_info),
    ]

_direct_ebuild_providers = rule(
    implementation = _direct_ebuild_providers_impl,
    attrs = dict(
        category = attr.string(mandatory = True),
        tarball = attr.label(mandatory = True, allow_single_file = True),
        runtime_deps = attr.label_list(providers = [BinaryPackageInfo]),
    ),
    provides = [BinaryPackageInfo, BinaryPackageSetInfo],
)
