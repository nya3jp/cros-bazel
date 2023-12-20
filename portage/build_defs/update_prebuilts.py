#!/usr/bin/env python3
# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Updates the bazelrc file containing prebuilts."""

import argparse
import fcntl
import io
import pathlib
from typing import Optional

from cros.bazel.portage.bin.metadata import metadata_pb2


def main(
    metadata_path: pathlib.Path,
    disk_cache: Optional[str],
    remote_cache: str,
    materialized: pathlib.Path,
):
    materialized.touch(exist_ok=True)

    workspace = pathlib.Path(__file__).resolve().parent.parent.parent.parent
    if not (workspace / "MODULE.bazel").exists():
        raise AssertionError("Unable to find workspace root")
    out = workspace / "prebuilts.bazelrc"

    metadata = metadata_pb2.Metadata()
    metadata.ParseFromString(metadata_path.read_bytes())

    if disk_cache:
        value = f"{disk_cache}/cas/{metadata.sha256[:2]}/{metadata.sha256}"
    else:
        value = f"cas://{remote_cache}/{metadata.sha256}/{metadata.size}"
    line = f"build:prebuilts --{metadata.label}_prebuilt={value}\n"

    with out.open(mode="w", encoding="utf-8") as f:
        # We rely here on the lock being released when the program exits.
        fcntl.flock(f, fcntl.LOCK_EX)
        # Open the file in append mode. We can't use regular append mode because
        # of the race condition where we open the file, someone else locks it
        # and writes to it, and we are now pointing to the *old* end, then we
        # get the lock.
        f.seek(0, io.SEEK_END)
        f.write(line)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--metadata",
        dest="metadata_path",
        type=pathlib.Path,
        help="A path to a file containing a serialized Metadata proto",
    )
    parser.add_argument(
        "--remote_cache",
        default="projects/chromeos-bot/instances/cros-rbe-nonrelease",
        help="The rbe instance for the remote cache",
    )
    parser.add_argument(
        "--disk_cache",
        default=None,
        help=(
            "If provided, reads prebuilts from the disk cache instead of the "
            "remote cache"
        ),
    )
    parser.add_argument(
        "--materialized",
        type=pathlib.Path,
        help="A materialized file to force bazel to not be lazy.",
    )
    args = parser.parse_args()
    main(**vars(args))
