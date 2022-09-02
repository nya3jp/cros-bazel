# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_skylib//lib:paths.bzl", "paths")

BinaryPackageInfo = provider(
    "Portage binary package info",
    fields = {
        "file": """
            File: A binary package file (.tbz2) of this package.
        """,
        "runtime_deps": """
            Depset[File]: Binary package files (.tbz2) to be installed when
            this package is required in run time.
            The depset always contains the binary package file of this package
            itself.
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
        "squashfs_files": """
            File[]: Squashfs image files (.squashfs).
            The order matters; the first image must be overlayed on top of the
            second image, and so on.
        """,
    },
)

def _workspace_root(label):
    return paths.join("..", label.workspace_name) if label.workspace_name else ""

def relative_path_in_package(file):
    owner = file.owner
    if owner == None:
        fail("File does not have an associated owner label")
    return paths.relativize(file.short_path, paths.join(_workspace_root(owner), owner.package))
