# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_skylib//lib:paths.bzl", "paths")

BinaryPackageInfo = provider(
    "Portage binary package info",
    fields = {
        "file": "File of a binary package file (.tbz2)",
        "build_target_deps": "Depset[File] of binary package files (.tbz2)",
        "runtime_deps": "Depset[File] of binary package files (.tbz2)",
    },
)

OverlayInfo = provider(
    "Portage overlay info",
    fields = {
        "squashfs_file": "File of a squashfs image (.squashfs)",
        "mount_path": "String of a path where the overlay is mounted",
    },
)

OverlaySetInfo = provider(
    "Portage overlay set info",
    fields = {
        "overlays": "OverlayInfo[]",
    },
)

SDKInfo = provider(
    "ChromiumOS SDK info",
    fields = {
        "board": "string",
        "squashfs_files": "File[] of squashfs images (.squashfs)",
    },
)

def _workspace_root(label):
    return paths.join("..", label.workspace_name) if label.workspace_name else ""

def relative_path_in_package(file):
    owner = file.owner
    if owner == None:
        fail("File does not have an associated owner label")
    return paths.relativize(file.short_path, paths.join(_workspace_root(owner), owner.package))
