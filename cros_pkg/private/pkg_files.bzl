# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# Contents of this file is copy-pasted from @rules_pkg//pkg:mappings.bzl.
# The only change we make is to rename _pkg_files_impl to pkg_files_impl to make
# it accessible from other .bzl files.

"""Package creation helper mapping rules.
This module declares Provider interfaces and rules for specifying the contents
of packages in a package-type-agnostic way.  The main rules supported here are
the following:
- `pkg_files` describes destinations for rule outputs
- `pkg_mkdirs` describes directory structures
- `pkg_mklink` describes symbolic links
- `pkg_filegroup` creates groupings of above to add to packages
Rules that actually make use of the outputs of the above rules are not specified
here.
"""

load("@bazel_skylib//lib:paths.bzl", "paths")
load("@rules_pkg//pkg:providers.bzl", "PackageFilesInfo")

# TODO(#333): strip_prefix module functions should produce unique outputs.  In
# particular, this one and `_sp_from_pkg` can overlap.
_PKGFILEGROUP_STRIP_ALL = "."

REMOVE_BASE_DIRECTORY = "\0"

def _sp_files_only():
    return _PKGFILEGROUP_STRIP_ALL

def _sp_from_pkg(path = ""):
    if path.startswith("/"):
        return path[1:]
    else:
        return path

def _sp_from_root(path = ""):
    if path.startswith("/"):
        return path
    else:
        return "/" + path

strip_prefix = struct(
    _doc = """pkg_files `strip_prefix` helper.  Instructs `pkg_files` what to do with directory prefixes of files.
    Each member is a function that equates to:
    - `files_only()`: strip all directory components from all paths
    - `from_pkg(path)`: strip all directory components up to the current
      package, plus what's in `path`, if provided.
    - `from_root(path)`: strip beginning from the file's WORKSPACE root (even if
      it is in an external workspace) plus what's in `path`, if provided.
    Prefix stripping is applied to each `src` in a `pkg_files` rule
    independently.
 """,
    files_only = _sp_files_only,
    from_pkg = _sp_from_pkg,
    from_root = _sp_from_root,
)

def pkg_attributes(
        mode = None,
        user = None,
        group = None,
        uid = None,
        gid = None,
        **kwargs):
    """Format attributes for use in package mapping rules.
    If "mode" is not provided, it will default to the mapping rule's default
    mode.  These vary per mapping rule; consult the respective documentation for
    more details.
    Not providing any of "user", "group", "uid", or "gid" will result in the package
    builder choosing one for you.  The chosen value should not be relied upon.
    Well-known attributes outside of the above are documented in the rules_pkg
    reference.
    This is the only supported means of passing in attributes to package mapping
    rules (e.g. `pkg_files`).
    Args:
      mode: string: UNIXy octal permissions, as a string.
      user: string: Filesystem owning user name.
      group: string: Filesystem owning group name.
      uid: int: Filesystem owning user id.
      gid: int: Filesystem owning group id.
      **kwargs: any other desired attributes.
    Returns:
      A value usable in the "attributes" attribute in package mapping rules.
    """
    ret = kwargs
    if mode:
        ret["mode"] = mode
    if user:
        ret["user"] = user
    if group:
        ret["group"] = group
    if uid != None:
        if type(uid) != type(0):
            fail('Got "' + str(uid) + '" instead of integer uid')
        ret["uid"] = uid
    if gid != None:
        if type(gid) != type(0):
            fail('Got "' + str(gid) + '" instead of integer gid')
        ret["gid"] = gid

    if user != None and user.isdigit() and uid == None:
        print("Warning: found numeric username and no uid, did you mean to specify the uid instead?")

    if group != None and group.isdigit() and gid == None:
        print("Warning: found numeric group and no gid, did you mean to specify the gid instead?")

    return json.encode(ret)

####
# Internal helpers
####

def _do_strip_prefix(path, to_strip, src_file):
    if to_strip == "":
        # We were asked to strip nothing, which is valid.  Just return the
        # original path.
        return path

    path_norm = paths.normalize(path)
    to_strip_norm = paths.normalize(to_strip) + "/"

    if path_norm.startswith(to_strip_norm):
        return path_norm[len(to_strip_norm):]
    elif src_file.is_directory and (path_norm + "/") == to_strip_norm:
        return ""
    else:
        # Avoid user surprise by failing if prefix stripping doesn't work as
        # expected.
        #
        # We already leave enough breadcrumbs, so if File.owner() returns None,
        # this won't be a problem.
        failmsg = "Could not strip prefix '{}' from file {} ({})".format(to_strip, str(src_file), str(src_file.owner))
        if src_file.is_directory:
            failmsg += """\n\nNOTE: prefix stripping does not operate within TreeArtifacts (directory outputs)
To strip the directory named by the TreeArtifact itself, see documentation for the `renames` attribute.
"""
        fail(failmsg)

# The below routines make use of some path checking magic that may difficult to
# understand out of the box.  This following table may be helpful to demonstrate
# how some of these members may look like in real-world usage:
#
# Note: "F" is "File", "FO": is "File.owner".

# | File type | Repo     | `F.path`                                                 | `F.root.path`                | `F.short_path`          | `FO.workspace_name` | `FO.workspace_root` |
# |-----------|----------|----------------------------------------------------------|------------------------------|-------------------------|---------------------|---------------------|
# | Source    | Local    | `dirA/fooA`                                              |                              | `dirA/fooA`             |                     |                     |
# | Generated | Local    | `bazel-out/k8-fastbuild/bin/dirA/gen.out`                | `bazel-out/k8-fastbuild/bin` | `dirA/gen.out`          |                     |                     |
# | Source    | External | `external/repo2/dirA/fooA`                               |                              | `../repo2/dirA/fooA`    | `repo2`             | `external/repo2`    |
# | Generated | External | `bazel-out/k8-fastbuild/bin/external/repo2/dirA/gen.out` | `bazel-out/k8-fastbuild/bin` | `../repo2/dirA/gen.out` | `repo2`             | `external/repo2`    |

def _owner(file):
    # File.owner allows us to find a label associated with a file.  While highly
    # convenient, it may return None in certain circumstances, which seem to be
    # primarily when bazel doesn't know about the files in question.
    #
    # Given that a sizeable amount of the code we have here relies on it, we
    # should fail() when we encounter this if only to make the rare error more
    # clear.
    #
    # File.owner returns a Label structure
    if file.owner == None:
        fail("File {} ({}) has no owner attribute; cannot continue".format(file, file.path))
    else:
        return file.owner

def _relative_workspace_root(label):
    # Helper function that returns the workspace root relative to the bazel File
    # "short_path", so we can exclude external workspace names in the common
    # path stripping logic.
    #
    # This currently is "../$LABEL_WORKSPACE_ROOT" if the label has a specific
    # workspace name specified, else it's just an empty string.
    #
    # TODO(nacl): Make this not a hack
    return paths.join("..", label.workspace_name) if label.workspace_name else ""

def _path_relative_to_package(file):
    # Helper function that returns a path to a file relative to its package.
    owner = _owner(file)
    return paths.relativize(
        file.short_path,
        paths.join(_relative_workspace_root(owner), owner.package),
    )

def _path_relative_to_repo_root(file):
    # Helper function that returns a path to a file relative to its workspace root.
    return paths.relativize(
        file.short_path,
        _relative_workspace_root(_owner(file)),
    )

def pkg_files_impl(ctx):
    # The input sources are already known.  Let's calculate the destinations...

    # Exclude excludes
    srcs = [f for f in ctx.files.srcs if f not in ctx.files.excludes]

    if ctx.attr.strip_prefix == _PKGFILEGROUP_STRIP_ALL:
        src_dest_paths_map = {src: paths.join(ctx.attr.prefix, src.basename) for src in srcs}
    elif ctx.attr.strip_prefix.startswith("/"):
        # Relative to workspace/repository root
        src_dest_paths_map = {src: paths.join(
            ctx.attr.prefix,
            _do_strip_prefix(
                _path_relative_to_repo_root(src),
                ctx.attr.strip_prefix[1:],
                src,
            ),
        ) for src in srcs}
    else:
        # Relative to package
        src_dest_paths_map = {src: paths.join(
            ctx.attr.prefix,
            _do_strip_prefix(
                _path_relative_to_package(src),
                ctx.attr.strip_prefix,
                src,
            ),
        ) for src in srcs}

    out_attributes = json.decode(ctx.attr.attributes)

    # The least surprising default mode is that of a normal file (0644)
    out_attributes.setdefault("mode", "0644")

    # Do file renaming
    for rename_src, rename_dest in ctx.attr.renames.items():
        # rename_src.files is a depset
        rename_src_files = rename_src.files.to_list()

        # Need to do a length check before proceeding.  We cannot rename
        # multiple files simultaneously.
        if len(rename_src_files) != 1:
            fail(
                "Target {} expands to multiple files, should only refer to one".format(rename_src),
                "renames",
            )

        src_file = rename_src_files[0]
        if src_file not in src_dest_paths_map:
            fail(
                "File remapping from {0} to {1} is invalid: {0} is not provided to this rule or was excluded".format(rename_src, rename_dest),
                "renames",
            )

        if rename_dest == REMOVE_BASE_DIRECTORY:
            if not src_file.is_directory:
                fail(
                    "REMOVE_BASE_DIRECTORY as a renaming target for non-directories is disallowed.",
                    "renames",
                )

            # REMOVE_BASE_DIRECTORY results in the contents being dropped into
            # place directly in the prefix path.
            src_dest_paths_map[src_file] = ctx.attr.prefix

        else:
            src_dest_paths_map[src_file] = paths.join(ctx.attr.prefix, rename_dest)

    # At this point, we have a fully valid src -> dest mapping in src_dest_paths_map.
    #
    # Construct the inverse of this mapping to pass to the output providers, and
    # check for duplicated destinations.
    dest_src_map = {}
    for src, dest in src_dest_paths_map.items():
        if dest in dest_src_map:
            fail("After renames, multiple sources (at least {0}, {1}) map to the same destination.  Consider adjusting strip_prefix and/or renames".format(dest_src_map[dest].path, src.path))
        dest_src_map[dest] = src

    return [
        PackageFilesInfo(
            dest_src_map = dest_src_map,
            attributes = out_attributes,
        ),
        DefaultInfo(
            # Simple passthrough
            files = depset(dest_src_map.values()),
        ),
    ]
