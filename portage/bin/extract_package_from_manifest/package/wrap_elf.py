# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Extracts metadata from elf files and wraps them in wrapper shell scripts."""

import argparse
import json
import logging
import os

from chromite.third_party import lddtree


# Make lddtree use an actual logging library instead of just printing.
lddtree.warn = logging.warning
lddtree.dbg = logging.debug


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--sysroot", help="The path to the sysroot")
    parser.add_argument(
        "--elf-files",
        nargs="*",
        help="The path to each of the elf files.",
    )
    parser.add_argument(
        "--ld-library-path",
        nargs="+",
        help="The path to an entry in LD_LIBRARY_PATH.",
    )

    args = parser.parse_args()
    sysroot = args.sysroot

    def normalize(path: str):
        """Returns an absolute path without the sysroot prefix."""
        return "/" + os.path.relpath(path, sysroot)

    ld_library_path = args.ld_library_path
    # This is the format used by lddtree.
    ldpaths = {
        "conf": [
            os.path.join(sysroot, os.path.relpath(p, "/"))
            for p in ld_library_path
        ],
        "env": [],
        "interp": [],
    }

    elf_files = {}
    for path in args.elf_files:
        parsed = lddtree.ParseELF(path, root=sysroot, ldpaths=ldpaths)
        path = normalize(path)
        interp = parsed["interp"]
        rpath = parsed["rpath"]
        runpath = parsed["runpath"]

        # Ignore statically linked elf files.
        if not interp:
            continue

        libs = {}
        for name, lib in parsed["libs"].items():
            lib_path = lib["path"]
            if lib_path is None:
                # TODO: Change this to actually raising an exception.
                # This current setup just moves the failure down the line to
                # runtime, when we try to load a file without the shared
                # library present. For now though, this allows us to get it
                # working for most binaries.
                logging.error(
                    "Unable to find library %s, which is required by %s in "
                    "the sysroot. Is there a transitive runtime dependency "
                    "from $(equery belongs %s) to $(equery belongs %s)?",
                    name,
                    path,
                    path,
                    name,
                )
            # Libraries generally add needs = interp, but we don't really care
            # about that.
            elif lib_path != interp:
                libs[name] = normalize(lib_path)

        used_lib_dirs = {os.path.dirname(p) for p in libs.values()}
        minified_library_path = [
            d for d in ld_library_path if d in used_lib_dirs
        ]

        elf_files[path] = dict(
            interp=normalize(interp),
            libs=libs,
            rpath=[normalize(r) for r in rpath],
            runpath=[normalize(r) for r in runpath],
        )

        lddtree.GenerateLdsoWrapper(
            root=sysroot,
            path=path,
            interp=normalize(interp),
            libpaths=rpath + minified_library_path + runpath,
        )

    print(json.dumps(elf_files, sort_keys=True, indent=2))


if __name__ == "__main__":
    main()
