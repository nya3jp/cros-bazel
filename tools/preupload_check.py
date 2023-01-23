#!/usr/bin/env python3
# Copyright 2023 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
"""A tool to perform preupload checks."""

import argparse
import os
import re
import subprocess
import sys
from typing import Optional

_SCRIPT_NAME = os.path.basename(__file__)

_BUG_RE = r"\nBug: ?([Nn]one|\d+)"
_TEST_FIELD_RE = r"\nTest: \S+"


def _get_commit_desc(commit: str) -> str:
    """Returns the full commit message of a commit."""
    return subprocess.run(["git", "log", "--format=%B", commit + "^!"],
                          capture_output=True,
                          check=True,
                          encoding='utf-8').stdout


def _check_change_has_test_field(commit: str) -> Optional[str]:
    """Check for a non-empty 'Test: ' field in the commit message."""
    if not re.search(_TEST_FIELD_RE, _get_commit_desc(commit)):
        return "Changelist description needs Test: field (after first line)"
    return None


def _check_change_has_bug_field(commit: str) -> Optional[str]:
    """Check for a correctly formatted 'Bug:' field in the commit message."""

    if not re.search(_BUG_RE, _get_commit_desc(commit)):
        return ("Changelist description needs Bug field (after first line):\n"
                "Examples:\n"
                "Bug: 9999 (for buganizer)\n"
                "Bug: None")

    TEST_BEFORE_BUG_RE = _TEST_FIELD_RE + r".*" + _BUG_RE
    if re.search(TEST_BEFORE_BUG_RE, _get_commit_desc(commit), re.DOTALL):
        return "The Bug: field must come before the Test: field.\n"

    return None


def main():
    """The main function."""
    arg_parser = argparse.ArgumentParser(prog=_SCRIPT_NAME)
    arg_parser.add_argument('--commit', help='The git commit to be checked.')
    args = arg_parser.parse_args()

    error = _check_change_has_bug_field(args.commit)
    if error:
        print(error, file=sys.stderr)
        sys.exit(1)

    error = _check_change_has_test_field(args.commit)
    if error:
        print(error, file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
