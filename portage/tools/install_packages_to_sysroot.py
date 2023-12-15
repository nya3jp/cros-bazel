#!/usr/bin/env python3
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""A tool to build and install packages to the sysroot."""

import argparse
import logging
import os
import subprocess


_SCRIPT_DIR = os.path.dirname(__file__)
_SCRIPT_NAME = os.path.basename(__file__)
_BAZEL_PATH = "/mnt/host/source/chromite/bin/bazel"


def main():
    """The entry point of the program."""
    logging.basicConfig(level=logging.INFO)

    # Dump envvars for debugging.
    # TODO(b/313796569): Remove this.
    for key in os.environ:
        logging.info("envvar: %s=%s", key, os.environ[key])

    arg_parser = argparse.ArgumentParser(prog=_SCRIPT_NAME)
    arg_parser.add_argument(
        "--board", required=True, help="The target board name."
    )
    arg_parser.add_argument(
        "package_names", nargs="+", help="Names of the packages to install"
    )
    args = arg_parser.parse_args()

    os.environ["BOARD"] = args.board

    # Generate chromeos-chrome prebuilt config.
    chrome_prebuilt_configs = []
    # Stop using chrome prebuilt to verify goma works (b/300218625).
    # try:
    #     chrome_prebuilt_configs = (
    #         subprocess.check_output(
    #             [
    #                 os.path.join(
    #                     _SCRIPT_DIR, "generate_chrome_prebuilt_config.py"
    #                 ),
    #                 "--no-lookback",
    #             ],
    #             encoding="utf-8",
    #         )
    #         .strip()
    #         .splitlines()
    #     )
    # except subprocess.CalledProcessError as e:
    #     logging.info("Could not find chromeos-chrome prebuilt: %s", e)
    logging.info("Chrome prebuilt configs are %s", chrome_prebuilt_configs)

    # Build packages and install lists.
    subprocess.check_call(
        [
            _BAZEL_PATH,
            "build",
            "--profile=/tmp/allpackages_command.profile.gz",
            # --keep_going to keep building packages even after a failure to
            # detect as many failure as possible on the CI builders.
            # We may need to delete this after launching Alchemy.
            "--keep_going",
            "--execution_log_binary_file=/tmp/allpackages_exec.log",
            "--noexecution_log_sort",
        ]
        + chrome_prebuilt_configs
        + [
            "@portage//target/%s:installed" % package_name
            for package_name in args.package_names
        ],
    )


if __name__ == "__main__":
    main()
