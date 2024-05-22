# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Manages your vscode IDE configuration.

Rebases the changes in the chromeos standard IDE configuration onto your
custom config.
"""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile

from rules_python.python.runfiles import runfiles


def merge(src: Path, dest: Path) -> None:
    """Merges the changes since the last time we saw src into dest.

    Args:
        src: The file to merge
        dest: The destination file to merge into.
    """
    assert src.is_file()
    if not dest.is_file():
        shutil.copy(src, dest)

    # To ensure that we only merge the changes since we last saw it,
    # we need to record the contents when we last saw it.
    merge_base = dest.parent / f".{dest.name}.merge_base"
    if not merge_base.is_file():
        shutil.copy(src, merge_base)

    src_content = src.read_bytes()
    merge_base_content = merge_base.read_bytes()

    if src_content != merge_base_content:
        dest_content = dest.read_bytes()
        if dest_content == merge_base_content:
            # No conflicts.
            print(f"Updating {dest}")
            shutil.copy(src, dest)
        else:
            print(f"Merging {src} with {dest}")
            with tempfile.NamedTemporaryFile() as f:
                result = Path(f.name)
                shutil.copy(merge_base, result)
                subprocess.run(
                    [
                        "code",
                        "--wait",
                        "--merge",
                        src,
                        dest,
                        merge_base,
                        result,
                    ],
                    check=True,
                )
                shutil.copy(result, dest)

        shutil.copy(src, merge_base)
    else:
        print(f"No changes to {dest}")


def main():
    ws = os.environ.get("BUILD_WORKSPACE_DIRECTORY", None)
    if not ws:
        raise ValueError(
            "Run this script using `bazel run //bazel/docs/ide:configure`"
        )
    out_dir = Path(ws).parent

    r = runfiles.Create()
    in_dir = Path(
        r.Rlocation("cros/bazel/docs/ide/config/chromiumos.code-workspace")
    ).parent
    assert in_dir.is_dir()

    for d, _, filenames in os.walk(in_dir, followlinks=True):
        d = Path(d)
        for f in filenames:
            out_f = out_dir / d.relative_to(in_dir) / f
            out_f.parent.mkdir(exist_ok=True, parents=True)
            merge(d / f, out_f)


if __name__ == "__main__":
    main()
