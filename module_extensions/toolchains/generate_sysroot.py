#!/usr/bin/env python3
# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Generates a bazel repository from a tarball."""

import pathlib
import shutil
import subprocess
import sys


DST = "usr/x86_64-cros-linux-gnu/"


def _fix_clang(out_dir: pathlib.Path):
    # Symlinks don't play nice with bazel.
    # Referring to version numbers directly is a pain because it makes uprevs
    # harder.

    # clang(++) is a symlink to clang(++)-16, so we strip the version numbers.
    clang = out_dir / "usr/bin/clang"
    clang_cpp = out_dir / "usr/bin/clang++"
    real_clang = clang.resolve()
    real_clang_cpp = clang_cpp.resolve()

    # Note: We strip "++" in the wrapper script so that it executes clang.elf.
    wrapper_content = real_clang_cpp.read_text(encoding="utf-8")
    real_clang_cpp.write_text(
        wrapper_content.replace("${base}.elf", "${base%++}.elf"),
        encoding="utf-8",
    )

    real_clang.rename(clang)
    real_clang_cpp.rename(clang_cpp)

    real_clang.with_suffix(".elf").rename(clang.with_suffix(".elf"))
    real_clang_cpp.with_suffix(".elf").unlink()


def main(out_dir: pathlib.Path, tarball: pathlib.Path):
    print("Unpacking tarball")
    untar = ["tar", "-xf", str(tarball), "-C", str(out_dir)]
    if shutil.which("pixz"):
        untar.append("-Ipixz")

    subprocess.run(untar, check=True)
    print("Unpacked tarball")

    out_dir = out_dir.resolve()

    _fix_clang(out_dir)

    subprocess.run(
        ["rsync", "--archive", f"--link-dest={DST}", DST, "."],
        check=True,
        cwd=out_dir,
    )


if __name__ == "__main__":
    main(
        out_dir=pathlib.Path(sys.argv[1]),
        tarball=pathlib.Path(sys.argv[2]),
    )
