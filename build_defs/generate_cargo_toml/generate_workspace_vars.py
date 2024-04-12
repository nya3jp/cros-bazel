# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Generates variables to fill the workspace Cargo.toml template with."""

import argparse
import json
import pathlib


def main(
    out: pathlib.Path, manifest: pathlib.Path, members: list[pathlib.Path]
):
    deps = manifest.read_text().split("\n[dependencies]\n")[1]
    out_members = []
    for member in members:
        # Strip the Cargo.toml off the end.
        out_members.append(member.parent.relative_to("bazel"))

    out.write_text(
        json.dumps(
            dict(
                members=sorted([str(member) for member in out_members]),
                deps=deps,
            )
        )
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True, type=pathlib.Path)
    parser.add_argument("--manifest", required=True, type=pathlib.Path)
    parser.add_argument(
        "--members", required=True, nargs="+", type=pathlib.Path
    )

    main(**vars(parser.parse_args()))
