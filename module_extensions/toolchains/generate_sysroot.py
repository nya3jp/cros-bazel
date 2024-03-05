#!/usr/bin/env python3
# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Generates a bazel repository from a tarball."""

import json
import os
import pathlib
import shutil
import subprocess
import sys


DST = "usr/x86_64-cros-linux-gnu/"


def main(out_dir: pathlib.Path, tarball: pathlib.Path):
    print("Unpacking tarball")
    untar = ["tar", "-xf", str(tarball), "-C", str(out_dir)]
    if shutil.which("pixz"):
        untar.append("-Ipixz")

    subprocess.run(untar, check=True)
    print("Unpacked tarball")

    out_dir = out_dir.resolve()

    subprocess.run(
        ["rsync", "--archive", f"--link-dest={DST}", DST, "."],
        check=True,
        cwd=out_dir,
    )

    symlinks = {}
    for d, _, paths in os.walk(out_dir, followlinks=False):
        for file in paths:
            path = pathlib.Path(d, file)
            # For now, treat directory symlinks as real directories and leave
            # them untouched. Bazel doesn't really do directory symlinks well,
            # so not sure what the correct behaviour should be.
            if path.is_symlink():
                target = pathlib.Path(d, file).resolve()
                if target.is_file():
                    name = str(path.relative_to(out_dir))
                    target_name = str(target.relative_to(out_dir))
                    symlinks["/" + name] = "/" + target_name
                    path.unlink()

    with (out_dir / "symlinks.bzl").open("w", encoding="utf-8") as f:
        f.write("SYMLINKS = ")
        json.dump(symlinks, f, indent=4, sort_keys=True)


if __name__ == "__main__":
    main(
        out_dir=pathlib.Path(sys.argv[1]),
        tarball=pathlib.Path(sys.argv[2]),
    )
