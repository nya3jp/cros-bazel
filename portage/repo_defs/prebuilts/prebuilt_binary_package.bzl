# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/portage/build_defs:binary_package.bzl", "binary_package")

visibility("//bazel/portage/repo_defs/prebuilts/...")

def prebuilt_binary_package(name, **kwargs):
    kwargs.setdefault("category", native.package_name().rsplit("/", 1)[1])
    kwargs.setdefault("package_name", name)

    # Bazel doesn't actually care about the versions unless you have multiple
    # versions of the same package within the transitive dependencies of
    # a BinaryPackageInfo.
    # For example, filter_package allows you to filter down to a specific
    # ExtractedBinaryPackageInfo. Usually, you would write:
    # `filter_package(package_name = "dev-lang/python")`
    # But in that specific example, because we have two versions of python
    # installed, it would complain that it can't uniquely identify a package.
    # Thus, we would need:
    # `filter_package(package_name = "dev-lang/python", version = "3.8")`
    kwargs.setdefault("version", "unknown")
    binary_package(name = name, **kwargs)
