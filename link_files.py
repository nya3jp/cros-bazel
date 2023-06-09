#!/usr/bin/env python3
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Creates symlinks needed to run Bazel."""

import argparse
import logging
import os
import sys


def _create_link(target: str, link_name: str) -> None:
    if os.path.lexists(link_name):
        if not os.path.islink(link_name):
            raise RuntimeError(
                "Unexpected file type found at %s" % os.path.abspath(link_name)
            )

        if os.readlink(link_name) != target:
            logging.info("Removing stale symlink %s", link_name)
            os.remove(link_name)

    if not os.path.lexists(link_name):
        logging.info("Creating symlink %s", link_name)
        os.symlink(target, link_name)


def main() -> int:
    logging.basicConfig(level=logging.INFO)

    arg_parser = argparse.ArgumentParser()
    arg_parser.add_argument(
        "--mode",
        choices=["alchemy", "metallurgy"],
        default="alchemy",
        help="The mode to run this script in",
    )
    args = arg_parser.parse_args()

    # Change the current directory to the root.
    root = os.path.realpath(__file__)
    for _ in range(2):
        root = os.path.dirname(root)
    os.chdir(root)

    # Create symlinks to files and directories under workspace_root directories.
    known_symlinks = [
        "bazel-bin",
        "bazel-out",
        "bazel-src",
        "bazel-testlogs",
    ]
    for path in [
        "bazel/workspace_root/general",
        f"bazel/workspace_root/{args.mode}",
    ]:
        for entry in os.listdir(path):
            _create_link(os.path.join(path, entry), entry)
            known_symlinks.append(entry)

    # Create a symlink to bazel/rules_cros.
    _create_link("bazel/rules_cros", "rules_cros")
    known_symlinks.append("rules_cros")

    # Remove unneeded symlinks.
    for entry in os.listdir("."):
        if os.path.islink(entry) and entry not in known_symlinks:
            logging.info("Removing stale symlink %s", entry)
            os.remove(entry)

    return 0


if __name__ == "__main__":
    sys.exit(main())
