# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load(":files.bzl", "ExtractedFilegroupInfo", "get_file_type")
load(":package.bzl", "ExtractedBinaryPackageDirectInfo", "ExtractedBinaryPackageInfo", "ExtractedBinaryPackageSetInfo")

visibility("//bazel/portage/build_defs")

def _filter_package_impl(ctx):
    all_packages = ctx.attr.interface[ExtractedBinaryPackageSetInfo].packages
    name = ctx.attr.package_name
    slot = ctx.attr.slot
    if slot:
        package = all_packages.get((name, slot), None)
    else:
        packages = [
            pkg
            for ((got_name, _), pkg) in all_packages.items()
            if got_name == name
        ]
        if len(packages) == 1:
            package = packages[0]
        elif len(packages) > 1:
            fail("Expected a single package with name %r. Got slots %r" % (
                name,
                [pkg.pkg.slot for pkg in packages],
            ))
        else:
            package = None

    if package == None:
        slot = repr(slot) if slot else "<any slot>"
        fail(
            "Unable to find package (%r, %s) in the following packages:\n%s" % (
                name,
                slot,
                "\n".join([repr(k) for k in sorted(all_packages.keys())]),
            ),
        )

    return [
        DefaultInfo(files = package.pkg.runfiles),
        package.pkg,
        package,
        ExtractedFilegroupInfo(files = {
            file.path: file
            for file in package.pkg.transitive_files.to_list()
        }),
    ]

filter_package = rule(
    implementation = _filter_package_impl,
    attrs = dict(
        interface = attr.label(mandatory = True, providers = [ExtractedBinaryPackageInfo]),
        package_name = attr.string(mandatory = True),
        slot = attr.string(),
        transitive = attr.bool(default = True),
    ),
    provides = [ExtractedBinaryPackageInfo, ExtractedBinaryPackageDirectInfo, ExtractedFilegroupInfo],
)

def _filter_file_type_impl(ctx):
    file_type = ctx.attr.file_type
    include_symlinks = ctx.attr.include_symlinks
    files = ctx.attr.interface[ExtractedFilegroupInfo].files.values()
    filtered = [
        file
        for file in files
        if get_file_type(file, file_type, include_symlinks) != None
    ]

    return [
        DefaultInfo(files = depset(transitive = [f.runfiles for f in filtered])),
        ExtractedFilegroupInfo(files = {file.path: file for file in filtered}),
    ]

filter_file_type = rule(
    implementation = _filter_file_type_impl,
    attrs = dict(
        interface = attr.label(mandatory = True, providers = [ExtractedFilegroupInfo]),
        file_type = attr.string(mandatory = True),
        include_symlinks = attr.bool(mandatory = True),
    ),
    provides = [ExtractedFilegroupInfo],
)

def _filter_paths_impl(ctx, exe_path = None):
    files = ctx.attr.interface[ExtractedFilegroupInfo].files
    filtered = []
    for path in ctx.attr.paths + ([] if exe_path == None else [exe_path]):
        if path not in files:
            fail("%s was not present in %s" % (path, ctx.attr.interface))
        filtered.append(files[path])

    if exe_path == None:
        exe = None
    else:
        exe = ctx.actions.declare_file(ctx.label.name)
        ctx.actions.symlink(
            output = exe,
            target_file = files[exe_path].file,
            is_executable = True,
        )

    return [
        DefaultInfo(
            files = depset(
                [] if exe == None else [exe],
                transitive = [f.runfiles for f in filtered],
            ),
            executable = exe,
        ),
        ExtractedFilegroupInfo(files = {file.path: file for file in filtered}),
    ]

filter_paths = rule(
    implementation = _filter_paths_impl,
    attrs = dict(
        interface = attr.label(mandatory = True, providers = [ExtractedFilegroupInfo]),
        paths = attr.string_list(mandatory = True),
    ),
    provides = [ExtractedFilegroupInfo],
)

def _filter_executable_impl(ctx):
    return _filter_paths_impl(ctx, exe_path = ctx.attr.executable)

filter_executable = rule(
    implementation = _filter_executable_impl,
    attrs = dict(
        interface = attr.label(mandatory = True, providers = [ExtractedFilegroupInfo]),
        paths = attr.string_list(default = []),
        executable = attr.string(mandatory = True),
    ),
    executable = True,
)
