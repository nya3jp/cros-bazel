# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_skylib//lib:paths.bzl", "paths")

BinaryPackageInfo = provider(
    "Portage binary package info",
    fields = {
        "file": """
            File: A binary package file (.tbz2) of this package.
        """,
        "transitive_runtime_deps_files": """
            Depset[File]: Binary package files (.tbz2) to be installed when
            this package is required in run time.

            The depset *always* contains the binary package file of this package
            itself.
        """,
        "transitive_runtime_deps_targets": """
            Depset[Target]: Transitive runtime targets to be installed when this
            package is required at run time.
        """,
        "direct_runtime_deps_targets": """
            list[Target]: Direct runtime targets
        """,
    },
)

OverlayInfo = provider(
    "Portage overlay info",
    fields = {
        "squashfs_file": """
            File: A squashfs image (.squashfs) that contains files of this
            overlay.
        """,
    },
)

OverlaySetInfo = provider(
    "Portage overlay set info",
    fields = {
        "overlays": """
            OverlayInfo[]: Overlays.
        """,
    },
)

SDKBaseInfo = provider(
    "ChromiumOS SDK",
    fields = {
        "layers": """
            File[]: A list of files each of which represents a file system layer
            of the SDK. A layer file can be a directory, a tar file (.tar or
            .tar.zst) or a squashfs image file (.squashfs).
            The order matters; the first image must be overlayed on top of the
            second image, and so on.
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
            of the SDK.  A layer file can be a directory, a tar file (.tar or
            .tar.zst) or a squashfs image file (.squashfs).
            The order matters; the first image must be overlayed on top of the
            second image, and so on.
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
