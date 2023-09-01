# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/portage/build_defs:common.bzl", "BinaryPackageSetInfo")

visibility("private")

EXTRACT_COMMON_ATTRS = dict(
    shared_library_dir_regexes = attr.string_list(
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
    patch_elf = attr.bool(
        default = True,
        doc = "Whether to patch elf files to run outside of the SDK.",
    ),
)

ExtractedPackageSetInfo = provider(
    fields = dict(
        packages = """Dict[(str, str), ExtractedPackageInfo]:
        A mapping from (package name, slot) to ExtractedPackageInfo""",
    ),
)

ExtractedPackageInfo = provider(
    fields = dict(
        pkg = "ExtractedPackageDirectInfo: pkg",
        transitive = "Depset[ExtractedPackageDirectInfo]",
    ),
)

ExtractedPackageDirectInfo = provider(
    fields = dict(
        name = "str: Name of the binpkg",
        version = "Option[str]: Version of the binpkg",
        uid = "str: Unique ID for the package",
        binpkg = "BinaryPackageInfo: The binary package for this file",
        files = "Depset[File]: The files contained within the package",
        runtime_deps = "tuple[ExtractedPackageInfo]",
        shared_libraries = "Depset[File]: The shared libraries for a package",
        shared_libraries_symlinks = "Depset[File]: Symlinks to shared libraries above.",
        shared_libraries_runfiles = "Depset[File]: The union of the two above fields.",
        header_files = "Depset[File]: The headers for a package.",
        runfiles = "Depset[File]: The union of all the above files.",
    ),
)
