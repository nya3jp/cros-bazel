# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load(":files.bzl", "get_extracted_files")

visibility("private")

ExtractedBinaryPackageSetInfo = provider(
    fields = dict(
        packages = """Dict[(str, str), ExtractedBinaryPackageInfo]:
        A mapping from (package name, slot) to ExtractedBinaryPackageInfo""",
    ),
)

def _extracted_package_info_init(*, pkg):
    transitive = depset(
        [pkg],
        transitive = [dep.transitive for dep in pkg.direct_deps],
    )
    return dict(pkg = pkg, transitive = transitive)

ExtractedBinaryPackageInfo, _new_extracted_package_info = provider(
    fields = dict(
        pkg = "ExtractedBinaryPackageDirectInfo: pkg",
        transitive = "Depset[ExtractedBinaryPackageDirectInfo]",
    ),
    init = _extracted_package_info_init,
)

def _extracted_package_direct_info_init(*, binpkg, files, owned_runfiles, direct_deps):
    runfiles = depset(transitive = [f.runfiles for f in files.to_list()])
    dep_runfiles = depset(transitive = [dep.pkg.runfiles for dep in direct_deps])
    transitive_files = depset(transitive = [dep.pkg.transitive_files for dep in direct_deps])
    transitive_runfiles = depset(transitive = [dep.pkg.transitive_runfiles for dep in direct_deps])
    return dict(
        uid = _binpkg_uid(binpkg),
        binpkg = binpkg,
        direct_deps = tuple(direct_deps),
        files = files,
        transitive_files = depset(transitive = [files, transitive_files]),
        runfiles = runfiles,
        transitive_runfiles = depset(transitive = [runfiles, transitive_runfiles]),
        owned_runfiles = tuple(owned_runfiles),
    )

ExtractedBinaryPackageDirectInfo, _new_extracted_package_direct_info = provider(
    fields = dict(
        uid = "(str, str): Unique ID for the package",
        binpkg = "BinaryPackageInfo: The binary package for this file",
        direct_deps = "tuple[ExtractedBinaryPackageInfo]: All direct dependencies of this package",
        files = "depset[ExtractedFileInfo]: The files contained within the package",
        transitive_files = "depset[ExtractedFileInfo]: The files contained within the transitive dependencies of the package",
        runfiles = "depset[File]: The runfiles for all files contained within the package.",
        transitive_runfiles = "depset[File]: The runfiles for all files contained within the package, and any transitive dependencies of the package.",
        owned_runfiles = "tuple[File]: The subset of runfiles owned by this particular package.",
    ),
    init = _extracted_package_direct_info_init,
)

def manifest_uid(pkg):
    return (pkg["name"], pkg["slot"])

def _binpkg_uid(pkg):
    return ("%s/%s" % (pkg.category, pkg.package_name), pkg.slot.split("/")[0])

def match_packages(binpkgs, manifest_pkgs, fail = fail):
    """Matches expected packages against the actual packages built.

    Returns entries in the order of binpkgs passed in as input.
    Fails if it finds a package in one that wasn't present in the other.

    Args:
      binary_packages: list[BinaryPackageInfo]: The binary packages to match.
      manifest_pkgs: list[dict[str, any]]: A list of manifest entries, each
       corresponding to a package.
      fail: A function to call if we were unable to find a match

    Returns list[(uid, BinaryPackageInfo, manifest_package)]
    """
    manifest_pkgs = {
        manifest_uid(pkg): pkg["content"]
        for pkg in manifest_pkgs
    }
    matches = []
    for package in binpkgs:
        uid = _binpkg_uid(package)
        manifest_entry = manifest_pkgs.pop(uid, None)
        if manifest_entry == None:
            fail("%s (slot %s) was produced by the package unexpectedly" % uid)
        matches.append((uid, package, manifest_entry))
    for uid in manifest_pkgs:
        fail("%s (slot %s) was not produced by the package" % uid)
    return matches

def generate_packages(ctx, binpkgs, manifest_pkgs, fail = fail):
    """Generates ExtractedBinaryPackageInfo from the BinaryPackageInfo and the manifest.

    Args:
      binpkgs: list[BinaryPackageInfo]: The binary packages to extract from.
      manifest_pkgs: list[dict[str, any]]: A list of manifest entries, each
        corresponding to a package.
      fail: The function to call on failure.

    Returns {uid: ExtractedBinaryPackageInfo}
    """
    matches = match_packages(
        binpkgs = binpkgs,
        manifest_pkgs = manifest_pkgs,
        fail = fail,
    )

    packages_by_path = {binpkg.file.path: binpkg for binpkg in binpkgs}
    packages_by_uid = {}
    all_files = {}
    for uid, binpkg, manifest_pkg in matches:
        content = manifest_pkg
        files, owned_runfiles = get_extracted_files(ctx, all_files, content)

        direct_deps = [
            packages_by_uid[_binpkg_uid(packages_by_path[dep.path])]
            for dep in binpkg.direct_runtime_deps
        ]

        direct_info = ExtractedBinaryPackageDirectInfo(
            binpkg = binpkg,
            files = files,
            owned_runfiles = owned_runfiles,
            direct_deps = direct_deps,
        )

        info = ExtractedBinaryPackageInfo(pkg = direct_info)
        packages_by_uid[uid] = info
    return packages_by_uid
