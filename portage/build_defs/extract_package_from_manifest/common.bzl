# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/portage/build_defs:common.bzl", "BinaryPackageSetInfo")

visibility("private")

EXTRACT_COMMON_ATTRS = dict(
    ld_library_path_regexes = attr.string_list(
        doc = "A list of regexes for directories containing shared libraries.",
    ),
    header_file_dir_regexes = attr.string_list(
        doc = "A list of regexes for directories transitively containing header files.",
    ),
    manifest_regenerate_command = attr.string(),
    pkg = attr.label(
        mandatory = True,
        providers = [BinaryPackageSetInfo],
        doc = "The binary package to extract from (including transitive deps)",
    ),
)
