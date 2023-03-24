# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_skylib//lib:paths.bzl", "paths")

BinaryPackageInfo = provider(
    """
    Describes a Portage binary package.

    A rule providing BinaryPackageInfo must also provide BinaryPackageSetInfo
    that covers all transitive runtime dependencies of this package.

    All fields in this provider must have immutable types (i.e. no list/dict)
    because a BinaryPackageInfo is always accompanied by a BinaryPackageSetInfo
    referencing it with a depset.
    """,
    fields = {
        "file": """
            File: A binary package file (.tbz2) of this package.
        """,
        "all_files": """
            Depset[File]: All binary package files including this package's one
                itself and all transitive runtime dependencies.
        """,
        "direct_runtime_deps": """
            tuple[BinaryPackageInfo]: Direct runtime dependencies of the
            package. See the provider description for why this field is a tuple,
            not a list.
        """,
        "transitive_runtime_deps": """
            Depset[BinaryPackageInfo]: Transitive runtime dependencies of the
                package. Note that this depset does *NOT* contain this package
                itself, just because it is impossible to construct a
                self-referencing provider.
        """,
    },
)

BinaryPackageSetInfo = provider(
    """
    Represents a set of Portage binary packages.

    A package set represented by this provider is always closed over transitive
    runtime dependencies. That is, if the set contains a package X, it also
    contains all transitive dependencies of the package X.

    A rule providing BinaryPackageInfo must also provide BinaryPackageSetInfo
    that covers all transitive runtime dependencies of this package.
    """,
    fields = {
        "packages": """
            Depset[BinaryPackageInfo]: All Portage binary packages included in
                this set.
        """,
        "files": """
            Depset[File]: All Portage binary package files included in this set.
        """,
    },
)

OverlaySetInfo = provider(
    "Portage overlay set info",
    fields = {
        "overlays": """
            PackageArtifactInfo[]: Overlays in .tar.zst format.
        """,
    },
)

SDKBaseInfo = provider(
    "ChromiumOS SDK",
    fields = {
        "layers": """
            File[]: A list of files each of which represents a file system layer
            of the SDK. A layer file can be a directory or a tar file (.tar or
            .tar.zst). Layers are ordered from lower to upper; in other words,
            a file from a layer can be overridden by one in another layer that
            appears later in the list.
        """,
    },
)

SDKInfo = provider(
    "ChromiumOS Board SDK info",
    fields = {
        "board": """
            str: A board name.
        """,
        "layers": """
            File[]: A list of files each of which represents a file system layer
            of the SDK. A layer file can be a directory or a tar file (.tar or
            .tar.zst). Layers are ordered from lower to upper; in other words,
            a file from a layer can be overridden by one in another layer that
            appears later in the list.
        """,
        "overlays": """
            OverlaySetInfo: The set of overlays that makeup the board. This will
            generally contain the overlay-<board> or overlay-<board>-private
            overlay and all the parents of that overlay.
        """,
    },
)

EbuildLibraryInfo = provider(
    "Ebuild library info",
    fields = {
        "strip_prefix": """
            str: The prefix to strip off the files when installing into the sdk.
        """,
        "headers": """
            Depset[File]: Headers provided by the package.
        """,
        "pkg_configs": """
            Depset[File]: .pc files provided by the package.
        """,
        "shared_libs": """
            Depset[File]: .so files provided by the package.
        """,
        "static_libs": """
            Depset[File]: .a files provided by the package.
        """,
    },
)

def _workspace_root(label):
    return paths.join("..", label.workspace_name) if label.workspace_name else ""

def relative_path_in_label(file, label):
    return paths.relativize(file.short_path, paths.join(_workspace_root(label), label.package))

def relative_path_in_package(file):
    owner = file.owner
    if owner == None:
        fail("File does not have an associated owner label")
    return relative_path_in_label(file, owner)

def single_binary_package_set_info(package_info):
    """
    Creates BinaryPackageSetInfo for a single binary package.

    Args:
        package_info: BinaryPackageInfo: A provider describing a binary package.

    Returns:
        BinaryPackageSetInfo: A provider representing all transitive runtime
            dependencies of the given binary package.
    """
    return BinaryPackageSetInfo(
        packages = depset(
            [package_info],
            transitive = [
                depset([dep], transitive = [dep.transitive_runtime_deps])
                for dep in package_info.direct_runtime_deps
            ],
        ),
        files = package_info.all_files,
    )
