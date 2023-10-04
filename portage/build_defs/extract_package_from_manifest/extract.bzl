# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/build_defs:rule_helpers.bzl", "get_output_dir")
load("//bazel/portage/build_defs:common.bzl", "BinaryPackageSetInfo")
load(":common.bzl", "EXTRACT_COMMON_ATTRS")
load(":files.bzl", "ExtractedFilegroupInfo", "SHARED_LIBRARY", "filter_files")
load(":package.bzl", "ExtractedBinaryPackageDirectInfo", "ExtractedBinaryPackageInfo", "ExtractedBinaryPackageSetInfo", "generate_packages", "manifest_uid")

visibility("//bazel/portage/build_defs")

def _extract_impl(ctx):
    args = ctx.actions.args()

    manifest = json.decode(ctx.attr.manifest_content)
    ld_library_path = manifest["ld_library_path"]
    header_file_dir_regexes = manifest["header_file_dir_regexes"]

    for path in ld_library_path:
        args.add("--ld-library-path", path)
    for d in header_file_dir_regexes:
        args.add("--header-file-dir-regex", d)
    args.add("--regenerate-command", ctx.attr.manifest_regenerate_command)
    args.add("--out-dir", get_output_dir(ctx))

    outputs = []

    def _requires_update(msg):
        # A change here requires a change to extract/src/main.rs and
        # .bazel_fix_commands.json.
        fail("%s\nInterface for binary has changed. Please run '%s'" %
             (msg, ctx.attr.manifest_regenerate_command))

    manifest_pkgs = manifest["packages"]
    for pkg in manifest_pkgs:
        for metadata in pkg["content"].values():
            # Default deserialization only works on structs, not on enums.
            # See https://serde.rs/container-attrs.html#default,
            metadata.setdefault("type", "Unknown")

    manifest_by_uid = {manifest_uid(pkg): pkg for pkg in manifest_pkgs}

    packages = generate_packages(
        ctx,
        binpkgs = ctx.attr.pkg[BinaryPackageSetInfo].packages.to_list(),
        manifest_pkgs = manifest_pkgs,
        fail = _requires_update,
    )

    all_files = {}
    for uid, info in packages.items():
        uid_pretty = "%s_slot_%s" % uid
        direct_info = info.pkg

        # Avoid creating actions for virtual packages.
        pkg_files = direct_info.files.to_list()
        if pkg_files:
            for file in pkg_files:
                all_files[file.path] = file
            outputs.append(direct_info.runfiles)

            pkg_args = ctx.actions.args()
            pkg_args.add("--binpkg", direct_info.binpkg.file)

            manifest_file_name = "%s_%s_manifest_content" % (
                ctx.label.name,
                uid_pretty.replace("/", "_"),
            )
            manifest_file = ctx.actions.declare_file(manifest_file_name)
            ctx.actions.write(manifest_file, json.encode(manifest_by_uid[uid]))
            pkg_args.add("--manifest", manifest_file)

            inputs = [manifest_file, direct_info.binpkg.file]

            # Gather the transitive dependencies of our direct deps, which
            # ensures we don't depend on ourself.
            transitive_files = depset(transitive = [
                f.pkg.transitive_files
                for f in direct_info.direct_deps
            ])

            # Ensure that we have shared libraries present before
            # attempting to extract binaries depending on them.
            shared_libs = depset(transitive = [
                f.runfiles
                for f in filter_files(transitive_files.to_list(), SHARED_LIBRARY)
            ])
            inputs.extend(shared_libs.to_list())

            ctx.actions.run(
                executable = ctx.executable._extract_interface,
                outputs = direct_info.owned_runfiles,
                inputs = inputs,
                arguments = [args, pkg_args],
                progress_message = "Extracting package %s" % uid_pretty,
            )

    root_package = packages[manifest_uid(manifest["root_package"])]
    return [
        DefaultInfo(files = depset(transitive = outputs)),
        ExtractedBinaryPackageSetInfo(packages = packages),
        root_package.pkg,
        root_package,
        ExtractedFilegroupInfo(files = all_files),
    ]

extract = rule(
    implementation = _extract_impl,
    attrs = EXTRACT_COMMON_ATTRS | dict(
        manifest_content = attr.string(mandatory = True),
        _extract_interface = attr.label(
            executable = True,
            default = "//bazel/portage/bin/extract_package_from_manifest/extract",
            cfg = "exec",
        ),
    ),
    provides = [
        ExtractedBinaryPackageSetInfo,
        ExtractedBinaryPackageInfo,
        ExtractedBinaryPackageDirectInfo,
        ExtractedFilegroupInfo,
    ],
)
