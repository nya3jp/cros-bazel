# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/cros_pkg/private:cros_pkg_filegroup.bzl", "dirs", "symlink", _cros_pkg_filegroup = "cros_pkg_filegroup", _custom_file_type = "custom_file_type", _unimplemented = "unimplemented")
load("//bazel/cros_pkg/private:pkg_files.bzl", _strip_prefix = "strip_prefix")

cros_pkg_filegroup = _cros_pkg_filegroup
custom_file_type = _custom_file_type
strip_prefix = _strip_prefix

# Each of these supports the following fields:
# * srcs: (List[Label|File]) The files to install.
# * name: (str) What to rename the file to.
# * prefix: (str) The directory to install into.
# * dst: (str) Equivalent to specifying both dst and name.
# * strip_prefix: (strip_prefix) How to strip the prefix. Options are:
#    - `strip_prefix.files_only()`: strip all directory components from all
#      paths
#    - `strip_prefix.from_pkg(path=None)`: strip all directory components up to
#      the current package, plus what's in `path`, if provided.
#    - `strip_prefix.from_root(path)`: strip beginning from the file's WORKSPACE
#      root (even if it is in an external workspace) plus what's in `path`, if
#      provided.
# * mode: (str) The file permissions (defaults to "0644").
# * user: (str) The name of the user owning the files.
# * group: (str) The name of the group owning the files.
# * uid: (str) The numeric user owner of the files.
# * gid: (str) The numeric group owner of the files.

# The way we calculate the file's destination is:
# * If dst is specified, use that path.
# * Otherwise, use prefix + name
#   * The prefix is calculated by:
#     * If it was explicitly specified, use that
#     * Try the default value for the file type (eg. /usr/bin for pkg.bin).
#     * The build will fail if neither of these are met.
#   * The name is calculated by:
#     * If it was explicitly specified, use that
#     * Otherwise, for each file, it looks at the path and strip_prefix:
#       * (default) strip_prefix.files_only(): $(basename path)
#       * strip_prefix.from_pkg(): path relative_to the current package
#       * strip_prefix.from_pkg("foo"): path relative_to the specified package
#       * strip_prefix.from_root(): path relative to the root

pkg = struct(
    symlink = symlink,
    dirs = dirs,
    bin = custom_file_type(
        mode = "0755",
        prefix = "/usr/bin",
        user = "root",
        group = "root",
        uid = 0,
        gid = 0,
    ),
    confd = custom_file_type(prefix = "/etc/conf.d"),
    doc = custom_file_type(prefix = "/usr/share/doc"),
    envd = custom_file_type(prefix = "/etc/env.d"),
    exe = custom_file_type(mode = "0755"),
    header = custom_file_type(prefix = "/usr/include"),
    html = _unimplemented("Use pkg.doc instead of pkg.html"),
    info = _unimplemented(),
    initd = custom_file_type(prefix = "/etc/init.d"),
    # Use file instead of ins.
    file = custom_file_type(),
    ins = _unimplemented("Use pkg.file instead of pkg.ins"),
    lib = _unimplemented(),
    lib_a = _unimplemented("Use pkg.lib instead of pkg.lib_a"),
    lib_so = _unimplemented("Use pkg.lib instead of pkg.lib_so"),
    man = custom_file_type(prefix = "/usr/share/man"),
    mo = _unimplemented(),
    sbin = _unimplemented(),
)
