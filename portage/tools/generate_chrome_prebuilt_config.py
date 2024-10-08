#!/usr/bin/env python3
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""A tool to generate bazel config to use a prebuilt for chromeos-chrome."""

import argparse
import logging
import os
import re
import subprocess
from typing import Dict, List


_SCRIPT_NAME = os.path.basename(__file__)


def _run_command(args: List[str]) -> str:
    """Runs the specified command and returns its output."""
    return subprocess.check_output(args, encoding="utf-8")


def _resolve_alias(label: str) -> List[str]:
    """Resolves the specified Bazel alias and returns its actual labels."""
    actuals = _run_command(
        ["bazel", "query", f"labels('actual', {label})"]
    ).splitlines()
    if len(actuals) == 0:
        return [label]
    return [l for actual in actuals for l in _resolve_alias(actual)]


def _get_chromeos_version_sh_path() -> str:
    """Returns the path of the chromeos_version.sh file."""
    return os.path.join(
        _run_command(["bazel", "info", "workspace"]).strip(),
        "third_party/chromiumos-overlay/chromeos/config/chromeos_version.sh",
    )


def _get_chromeos_version_info() -> Dict[str, str]:
    """Returns ChromeOS version info."""
    result = {}
    for line in _run_command([_get_chromeos_version_sh_path()]).split("\n"):
        m = re.match(r"(\w+)=(.*)", line.strip())
        if m:
            result[m.group(1)] = m.group(2)
    return result


def _find_chrome_prebuilt(
    board: str, branch: int, build: int, lookback: bool
) -> str:
    """Finds a chromeos-chrome prebuilt and returns its URL."""

    # Try up to 10 versions if lookback==True.
    for _ in range(10 if lookback else 1):
        logging.info(
            "Trying to find a prebuilt: branch = %i, build = %i", branch, build
        )
        try:
            output = _run_command(
                [
                    "gsutil",
                    "ls",
                    f"gs://chromeos-prebuilt/board/{board}/postsubmit-R{branch}-{build}*/packages/chromeos-base/chromeos-chrome*.tbz2",
                ]
            )
            return output.strip().splitlines()[-1]
        except subprocess.CalledProcessError:
            # Failed to find a usable prebuilt. Try an older version.
            build = build - 1
    raise RuntimeError("Failed to find a prebuilt.")


def main():
    """The entry point of the program."""

    logging.basicConfig(level=logging.INFO)

    arg_parser = argparse.ArgumentParser(prog=_SCRIPT_NAME)
    arg_parser.add_argument(
        "--lookback",
        action="store_true",
        default=True,
        dest="lookback",
        help="Look back for old version prebuilts.",
    )
    arg_parser.add_argument(
        "--no-lookback",
        action="store_false",
        dest="lookback",
        help="Do not look back for old version prebuilts.",
    )
    args = arg_parser.parse_args()

    board = os.environ["BOARD"]
    logging.info("Board name is %s", board)

    version_info = _get_chromeos_version_info()
    branch = int(version_info["CHROME_BRANCH"])
    build = int(version_info["CHROMEOS_BUILD"])
    logging.info(
        "Version numbers taken from the checkout: branch = %i, build = %i",
        branch,
        build,
    )

    chrome_label = "@portage//target/chromeos-base/chromeos-chrome"
    chrome_actual_labels = _resolve_alias(chrome_label)
    prebuilt_labels = [label + "_prebuilt" for label in chrome_actual_labels]
    logging.info("Prebuilt labels are %s", prebuilt_labels)

    prebuilt_url = _find_chrome_prebuilt(board, branch, build, args.lookback)

    for label in prebuilt_labels:
        print(f"--{label}={prebuilt_url}")


if __name__ == "__main__":
    main()
