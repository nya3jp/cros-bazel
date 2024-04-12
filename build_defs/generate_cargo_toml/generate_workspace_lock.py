# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Generates a Cargo.lock file for a fake workspace.

This reads the real Cargo.toml / Cargo.lock used by bazel, and uses them to
generate a fake Cargo.lock. This fake lockfile should have all the same 3p deps
as the real one, but simply adds extra 1p entries.
"""

import argparse
import pathlib
import shutil
import subprocess
import sys
import tempfile

from python.runfiles import runfiles


def _ensure_dir_exists(p: pathlib.Path):
    p.parent.mkdir(parents=True, exist_ok=True)
    return p


def main(
    out: pathlib.Path,
    lockfile: pathlib.Path,
    root_manifest: pathlib.Path,
    manifests: pathlib.Path,
    srcs: list[pathlib.Path],
):
    r = runfiles.Create()
    cargo = r.Rlocation("rust_host_tools/bin/cargo")

    with tempfile.TemporaryDirectory() as d:
        d = pathlib.Path(d)
        new_lockfile = d / "bazel/Cargo.lock"
        shutil.copy(lockfile, _ensure_dir_exists(new_lockfile))
        shutil.copy(root_manifest, d / "bazel/Cargo.toml")
        for path in manifests:
            _ensure_dir_exists((d / path)).symlink_to(path.resolve())
        for src in srcs:
            _ensure_dir_exists(d / src).touch()

        ps = subprocess.run(
            [cargo, "update", "--offline", "--workspace"],
            cwd=new_lockfile.parent,
            stderr=subprocess.PIPE,
            stdout=subprocess.PIPE,
            check=False,
        )

        if ps.returncode != 0:
            print("STDOUT", ps.stdout, file=sys.stdout)
            print("STDERR", ps.stderr, file=sys.stderr)
            sys.exit(ps.returncode)

        shutil.copy(new_lockfile, out)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True, type=pathlib.Path)
    parser.add_argument("--lockfile", required=True, type=pathlib.Path)
    parser.add_argument("--root_manifest", required=True, type=pathlib.Path)
    parser.add_argument(
        "--manifests", required=True, nargs="+", type=pathlib.Path
    )
    parser.add_argument("--srcs", required=True, nargs="+", type=pathlib.Path)

    main(**vars(parser.parse_args()))
