#!/usr/bin/env python3
# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Generates metadata for a portage package."""

import hashlib
import pathlib
import sys

from cros.bazel.portage.bin.metadata import metadata_pb2


def main(label: str, tbz2: pathlib.Path, out: pathlib.Path):
    metadata = metadata_pb2.Metadata()
    metadata.label = label
    metadata.size = tbz2.stat().st_size
    with tbz2.open("rb") as f:
        metadata.sha256 = hashlib.file_digest(f, "sha256").hexdigest()
    out.write_bytes(metadata.SerializeToString())


if __name__ == "__main__":
    # Usage: gen_metadata \
    #   @portage//foo/bar \
    #   path/to/bar.tbz2 \
    #   path/to/bar_metadata.binaryproto
    main(sys.argv[1], pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3]))
