#!/usr/bin/env python3
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""A tool to build and install packages to the sysroot."""

import argparse
import json
import logging
import os
import re
import subprocess
from typing import List


_SCRIPT_DIR = os.path.dirname(__file__)
_SCRIPT_NAME = os.path.basename(__file__)
_BAZEL_PATH = "/mnt/host/source/chromite/bin/bazel"

_BUILD_EVENT_JSON_FILE_PATH = "/tmp/chromeos_bazel_build_events.json"


def _GetFailedPackages(build_event_json_file: str) -> List[str]:
    """Reads the specified file and returns a list of failed packages.

    Each line of the input file is a JSON object which represents an event, and
    each event looks like this:
    {
      "id": {
        "actionCompleted": {
          "primaryOutput": ".../chrome-icu/chrome-icu-122.0.6226.0_rc-r1.tbz2",
          "label": "@@.../chromeos-base/chrome-icu:122.0.6226.0_rc-r1",
          "configuration": {
            "id": "..."
          }
        }
      },
      "action": {
        "exitCode": 1,
        "stderr": {
          "name": "stderr",
          "uri": "bytestream://remotebuildexecution.googleapis.com/..."
        },
        "label": "@@.../chromeos-base/chrome-icu:122.0.6226.0_rc-r1",
        "configuration": {
          "id": "..."
        },
        "type": "Ebuild",
        "commandLine": [
          ...
        ],
        "failureDetail": {
          "message": "local spawn failed for Ebuild",
          "spawn": {
            "code": "NON_ZERO_EXIT",
            "spawnExitCode": 1
          }
        }
      }
    }
    """
    failed_packages = set()
    with open(build_event_json_file, encoding="utf-8") as f:
        for line in f:
            # Look for actionCompleted events with failureDetails.
            event = json.loads(line)
            action_completed = event.get("id", {}).get("actionCompleted")
            failure_detail = event.get("action", {}).get("failureDetail")
            if action_completed and failure_detail:
                label = action_completed.get("label", "")
                m = re.match(
                    "@@_main~portage~portage//.*/([^/]+/[^:]+):", label
                )
                if m:
                    failed_packages.add(m.group(1))
    return list(failed_packages)


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
    try:
        subprocess.check_call(
            [
                _BAZEL_PATH,
                "build",
                "--profile=/tmp/allpackages_command.profile.gz",
                "--experimental_profile_include_target_label",
                "--experimental_profile_include_primary_output",
                # --keep_going to keep building packages even after a failure to
                # detect as many failure as possible on the CI builders.
                # We may need to delete this after launching Alchemy.
                "--keep_going",
                "--execution_log_binary_file=/tmp/allpackages_exec.log",
                "--noexecution_log_sort",
                "--config=hash_tracer",
                "--build_event_json_file=%s" % _BUILD_EVENT_JSON_FILE_PATH,
            ]
            + chrome_prebuilt_configs
            + [
                "@portage//target/%s:installed" % package_name
                for package_name in args.package_names
            ],
        )
    except subprocess.CalledProcessError:
        cros_metrics_dir = os.environ.get("CROS_METRICS_DIR")
        if cros_metrics_dir:
            failed_packages = _GetFailedPackages(_BUILD_EVENT_JSON_FILE_PATH)
            if failed_packages:
                with open(
                    os.path.join(cros_metrics_dir, "FAILED_PACKAGES"),
                    "w",
                    encoding="utf-8",
                ) as f:
                    for package in failed_packages:
                        # "unknown" is a place holder for the failing ebuild
                        # phase name which won't be used.
                        f.write("%s unknown\n" % package)
        raise


if __name__ == "__main__":
    main()
