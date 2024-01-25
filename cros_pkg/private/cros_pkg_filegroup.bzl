# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@rules_pkg//pkg:mappings.bzl", _strip_prefix = "strip_prefix")
load("@rules_pkg//pkg:providers.bzl", "PackageDirsInfo", "PackageFilegroupInfo", "PackageFilesInfo", "PackageSymlinkInfo")
load(":deploy.bzl", "deploy_local")
load(":pkg_files.bzl", "pkg_attributes", "pkg_files_impl")

visibility("//bazel/cros_pkg/...")

_KIND_FILE = "file"
_KIND_SYMLINK = "symlink"
_KIND_DIR = "dir"

def _gen_fake_ctx(ctx, src, metadata):
    """Generates a fake ctx-like object suitable for use with pkg_files_impl."""
    metadata = json.decode(metadata)
    attributes = pkg_attributes(
        mode = metadata.get("mode", None),
        user = metadata.get("user", None),
        group = metadata.get("group", None),
        uid = metadata.get("uid", None),
        gid = metadata.get("gid", None),
    )
    strip_prefix = metadata.get("strip_prefix", _strip_prefix.files_only())

    renames = {}
    if "name" in metadata:
        renames[src] = metadata["name"]

    return struct(
        files = struct(
            srcs = src.files.to_list(),
            excludes = [],
        ),
        attr = struct(
            strip_prefix = strip_prefix,
            prefix = metadata["prefix"],
            attributes = attributes,
            renames = renames,
        ),
    )

def _cros_pkg_filegroup_impl(ctx):
    files = []

    pkg_files = []
    pkg_dirs = []
    pkg_symlinks = []
    for target in ctx.attr.include:
        if PackageFilesInfo in target:
            pkg_files.append((target[PackageFilesInfo], target.label))
        if PackageDirsInfo in target:
            pkg_dirs.append((target[PackageDirsInfo], target.label))
        if PackageSymlinkInfo in target:
            pkg_symlinks.append((target[PackageSymlinkInfo], target.label))
        if PackageFilegroupInfo in target:
            provider = target[PackageFilegroupInfo]
            pkg_dirs.extend(provider.pkg_dirs)
            pkg_files.extend(provider.pkg_files)
            pkg_symlinks.extend(provider.pkg_symlinks)

        files.append(target[DefaultInfo].files)

    for symlink in json.decode(ctx.attr.symlinks):
        provider = PackageSymlinkInfo(
            destination = symlink["link_name"],
            target = symlink["target"],
            attributes = symlink["attributes"],
        )
        pkg_symlinks.append((provider, ctx.label))

    for d in json.decode(ctx.attr.dirs):
        provider = PackageDirsInfo(
            dirs = d["dirs"],
            attributes = d["attributes"],
        )
        pkg_dirs.append((provider, ctx.label))

    for src, metadata in ctx.attr.srcs.items():
        if PackageFilegroupInfo in src or PackageFilesInfo in src or PackageDirsInfo in src or PackageSymlinkInfo in src:
            fail("{} has unexpected provider. Use include".format(
                str(src.label),
            ))
        package_files_info, default_info = pkg_files_impl(
            _gen_fake_ctx(ctx, src, metadata),
        )
        files.append(default_info.files)
        pkg_files.append((package_files_info, src.label))

    return [
        DefaultInfo(
            files = depset(transitive = files),
        ),
        PackageFilegroupInfo(
            pkg_dirs = pkg_dirs,
            pkg_files = pkg_files,
            pkg_symlinks = pkg_symlinks,
        ),
    ]

_cros_pkg_filegroup = rule(
    implementation = _cros_pkg_filegroup_impl,
    attrs = dict(
        # Input file -> json-encoded metadata for output.
        srcs = attr.label_keyed_string_dict(allow_files = True),
        # Json-encoded metadata.
        symlinks = attr.string(mandatory = True),
        # Json-encoded metadata.
        dirs = attr.string(mandatory = True),
        include = attr.label_list(providers = [[PackageFilesInfo], [PackageDirsInfo], [PackageSymlinkInfo], [PackageFilegroupInfo]]),
    ),
)

def cros_pkg_filegroup(name, srcs = [], visibility = None, include = [], **kwargs):
    # Bazel doesn't support arbitrary types like Dict[string, List[Label]].
    # So we're stuck with converting this to a "label_keyed_string_dict".
    label_keyed_srcs = {}

    symlinks = []
    dirs = []

    for src in srcs:
        kind = getattr(src, "kind", None)

        if kind == _KIND_FILE:
            if len(src.srcs) > 1 and "name" in src.kwargs:
                fail((
                    "Cannot rename a file when multiple files are " +
                    "specified at once (in {})"
                ).format(name))

            for label in src.srcs:
                if label in label_keyed_srcs:
                    fail((
                        "{} is specified twice in the pkg's filegroup. If " +
                        "this is intended, consider splitting one of them " +
                        "out into a rules_pkg invocation directly, and " +
                        "adding pkg_files = [...] to the cros_pkg_filegroup " +
                        "invocation."
                    ).format(label))
                label_keyed_srcs[label] = src.kwargs
        elif kind == _KIND_SYMLINK:
            symlinks.append(src)
        elif kind == _KIND_DIR:
            dirs.append(src)
        else:
            fail(
                "Each entry in srcs should be an object generated by bazel " +
                "package helper functions (eg. pkg.bin, pkg.exe, pkg.file, " +
                "pkg.doc, pkg.symlink, pkg.dirs).",
            )

    _cros_pkg_filegroup(
        name = name,
        srcs = label_keyed_srcs,
        symlinks = json.encode(symlinks),
        dirs = json.encode(dirs),
        include = include,
        visibility = visibility,
        **kwargs
    )

    deploy_local(
        name = name + "_deploy_local",
        filegroup = name,
        visibility = visibility,
    )

def custom_file_type(**defaults):
    def fn(srcs, **kwargs):
        kwargs = defaults | kwargs
        if "dst" in kwargs:
            if "name" in kwargs or "prefix" in kwargs or "strip_prefix" in kwargs:
                fail("Name, prefix, and strip_prefix are incompatible with dst")
            kwargs["prefix"], kwargs["name"] = kwargs["dst"].rsplit("/", 1)
            kwargs["strip_prefix"] = _strip_prefix.files_only()

        if "prefix" not in kwargs:
            fail(
                "Unable to determine destination for {}. Try using dst or prefix",
            ).format(repr(srcs))

        return struct(
            kind = "file",
            srcs = srcs,
            kwargs = json.encode(kwargs),
        )

    return fn

def symlink(
        link_name,
        target,
        mode = "0640",
        user = None,
        group = None,
        uid = None,
        gid = None,
        **kwargs):
    if not link_name.startswith("/"):
        fail("Link name must be an absolute path to the symlink")
    return struct(
        kind = "symlink",
        target = target,
        link_name = link_name,
        attributes = json.decode(pkg_attributes(
            mode = mode,
            user = user,
            group = group,
            uid = uid,
            gid = gid,
            **kwargs
        )),
    )

def dirs(
        dirs,
        mode,
        user = None,
        group = None,
        uid = None,
        gid = None,
        **kwargs):
    return struct(
        kind = "dir",
        dirs = dirs,
        attributes = json.decode(pkg_attributes(
            mode = mode,
            user = user,
            group = group,
            uid = uid,
            gid = gid,
            **kwargs
        )),
    )

def unimplemented(message = "Not implemented yet"):
    def fn(srcs, **kwargs):
        fail(message)

    return fn
