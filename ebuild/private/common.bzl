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
        "mount_path": """
            str: A path where the overlay is mounted. It is a relative path
            from /mnt/host/source.
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

SDKInfo = provider(
    "ChromiumOS SDK info",
    fields = {
        "board": """
            str: A board name.
        """,
        "layers": """
            File[]: A list of files each of which represents a file system layer
            of the SDK. A layer file can be a directory, a symbolic link index
            file (.symindex), or a squashfs image file (.squashfs).
            The order matters; the first image must be overlayed on top of the
            second image, and so on.
        """,
    },
)

EbuildSrcInfo = provider(
    "Source files used by an ebuild",
    fields = {
        "file": """
            File: A .squashfs or tar.* that contains src files for this
            ebuild.
        """,
        "mount_path": """
            str: The patch where the src files will be mounted. It is relative
            to /mnt/host/source.
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

