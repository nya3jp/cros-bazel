# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/portage/build_defs:common.bzl", "BinaryPackageInfo")
load("//bazel/portage/build_defs:test_helpers.bzl", "assert_eq")
load(":files.bzl", "ELF_BINARY", "SHARED_LIBRARY", "SYMLINK")
load(":package.bzl", "generate_packages", "match_packages")

visibility("private")

_SHARED_LIB_PATH = "/lib64/libc.so.6"
_SYMLINK_PATH = "/lib64/libc.so"
_INTERP_PATH = "/lib64/ld-linux-x86-64.so.2"
_BINARY_PATH = "/usr/bin/nano"

_GLIBC_CONTENT = {
    _SHARED_LIB_PATH: {"type": SHARED_LIBRARY},
    "/lib64/libfoo.so.6": {"type": SHARED_LIBRARY},
    _SYMLINK_PATH: {
        "target": _SHARED_LIB_PATH,
        "type": SYMLINK,
    },
    _INTERP_PATH: {"type": SHARED_LIBRARY},
}
_NCURSES_CONTENT = {}
_NANO_CONTENT = {
    _BINARY_PATH: {
        "libs": {"libc.so.6": _SYMLINK_PATH},
        "type": ELF_BINARY,
    },
}

_PACKAGES = [
    {
        "content": _NANO_CONTENT,
        "name": "app-editors/nano",
        "slot": "0",
    },
    {
        "content": _GLIBC_CONTENT,
        "name": "sys-libs/glibc",
        "slot": "2.2",
    },
    {
        "content": _NCURSES_CONTENT,
        "name": "sys-libs/ncurses",
        "slot": "0",
    },
]

_GLIBC_UID = ("sys-libs/glibc", "2.2")
_NCURSES_UID = ("sys-libs/ncurses", "0")
_NANO_UID = ("app-editors/nano", "0")

def _test_package_impl(ctx):
    glibc_binpkg = ctx.attr._glibc[BinaryPackageInfo]
    ncurses_binpkg = ctx.attr._ncurses[BinaryPackageInfo]
    nano_binpkg = ctx.attr._nano[BinaryPackageInfo]

    # Ordering matters here, due to dependencies.
    binpkgs = [glibc_binpkg, ncurses_binpkg, nano_binpkg]

    assert_eq(
        match_packages(binpkgs, manifest_pkgs = _PACKAGES),
        [
            (_GLIBC_UID, glibc_binpkg, _GLIBC_CONTENT),
            (_NCURSES_UID, ncurses_binpkg, _NCURSES_CONTENT),
            (_NANO_UID, nano_binpkg, _NANO_CONTENT),
        ],
    )

    packages = generate_packages(ctx, binpkgs, _PACKAGES)
    glibc = packages[_GLIBC_UID].pkg
    ncurses = packages[_NCURSES_UID].pkg
    nano = packages[_NANO_UID].pkg

    assert_eq(packages[_GLIBC_UID].transitive.to_list(), [glibc])
    assert_eq(packages[_NCURSES_UID].transitive.to_list(), [glibc, ncurses])
    assert_eq(packages[_NANO_UID].transitive.to_list(), [glibc, ncurses, nano])

    assert_eq(len(glibc.runfiles.to_list()), 4)
    assert_eq(len(glibc.files.to_list()), 4)

    assert_eq(ncurses.uid, _NCURSES_UID)
    assert_eq(ncurses.binpkg, ncurses_binpkg)
    assert_eq(list(ncurses.direct_deps), [packages[_GLIBC_UID]])
    assert_eq(ncurses.files.to_list(), [])
    assert_eq(ncurses.transitive_files.to_list(), glibc.files.to_list())
    assert_eq(ncurses.runfiles.to_list(), [])
    assert_eq(ncurses.transitive_runfiles.to_list(), glibc.runfiles.to_list())

    assert_eq([f.path for f in nano.files.to_list()], ["/usr/bin/nano"])
    assert_eq(sorted([f.basename for f in nano.runfiles.to_list()]), [
        "ld-linux-x86-64.so.2",
        "libc.so",
        "libc.so.6",
        "nano",
        "nano.elf",
    ])

    # Ensure bazel doesn't complain that these files aren't generated.
    for f in nano.transitive_runfiles.to_list():
        ctx.actions.write(f, "")

test_package = rule(
    implementation = _test_package_impl,
    attrs = dict(
        _glibc = attr.label(default = "@files//:testdata_glibc_alias", providers = [BinaryPackageInfo]),
        _ncurses = attr.label(default = "@files//:testdata_ncurses_alias", providers = [BinaryPackageInfo]),
        _nano = attr.label(default = "@files//:testdata_nano_alias", providers = [BinaryPackageInfo]),
    ),
)
