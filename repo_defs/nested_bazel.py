#!/usr/bin/env python3
# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Runs a nested bazel invocation and collects the outputs to out_dir."""

import json
import logging
import os
import pathlib
import subprocess
import sys
from typing import List


def _fail(msg: str):
    print(msg, file=sys.stderr)
    # When invoked by the nested bazel repo rules, this will be enabled, and
    # thus it will tell you to run the binary directly. This adds colors to your
    # error messages (eg. compilation errors), and improves performance by
    # bypassing the outer bazel.
    if os.environ.get("SHOW_REPRO", None):
        cmd = " ".join(sys.argv)
        print(f"\n\nTo reproduce, run:\n{cmd}\n\n", file=sys.stderr)
    sys.exit(1)


def _run(args, **kwargs) -> subprocess.CompletedProcess:
    logging.debug("Running %s", " ".join(args))
    ps = subprocess.run(
        args,
        check=False,
        **kwargs,
        env={**os.environ, "IS_NESTED_BAZEL": "1"},
    )
    if ps.returncode != 0:
        # The command-line is really long and not particularly useful.
        _fail("Nested bazel invocation failed to execute.")
    return ps


def main(
    base_command: List[str],
    common_opts: List[str],
    build_opts: List[str],
    target: str,
    out_dir: str,
    repo_rule_deps: List[str],
    nested_output_base: str,
):
    # Eg. ~/.cache/_bazel_$USER_nested/<checksum>/external/a~b~c.
    out_dir = pathlib.Path(out_dir)
    out_root = pathlib.Path(nested_output_base, "execroot/_main")

    external_dir = out_dir.parent
    if external_dir.parent.name.endswith("_nested"):
        _fail(
            "You appear to be attempting to build a nested target from "
            "another nested target. This is not currently supported"
        )

    for repo_rule in repo_rule_deps:
        repo_path = external_dir.joinpath(repo_rule)
        common_opts.append(f"--override_repository={repo_rule}={repo_path}")

    _run(
        [
            *base_command,
            "build",
            *common_opts,
            *build_opts,
            target,
        ]
    )

    exported_files = []
    out_files = (
        _run(
            # This should get a cache hit because we already built everything.
            [
                *base_command,
                "cquery",
                "--output=files",
                *common_opts,
                *build_opts,
                target,
            ],
            stdout=subprocess.PIPE,
        )
        .stdout.decode("utf-8")
        .strip()
    )
    for path in out_files.split("\n"):
        path = out_root / path
        dst = out_dir / path.name
        exported_files.append(path.name)
        logging.debug("Symlinking %s to %s", dst, path)
        dst.symlink_to(path)

    with out_dir.joinpath("BUILD.bazel").open("w", encoding="utf-8") as build:
        build.write("exports_files([\n")
        for name in sorted(exported_files):
            build.write(f'    "{name}",\n')
        build.write("])\n")


if __name__ == "__main__":
    level = os.environ.get("NESTED_LOGGING", "").upper() or logging.INFO
    logging.basicConfig(level=level)

    with open(sys.argv[1], encoding="utf-8") as f:
        main(**(json.load(f)))
