#!/usr/bin/env python3
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Fake implementation of portageq used in unit tests."""

import shlex
import sys
from typing import List


_FAKE_DATA = {
    "/": {
        "chromiumos": "/mnt/host/source/src/third_party/chromiumos-overlay",
        "portage-stable": "/mnt/host/source/src/third_party/portage-stable",
    },
    "/build/meow/": {
        "meow": "/mnt/host/source/src/overlays/overlay-meow",
        "chromiumos": "/mnt/host/source/src/third_party/chromiumos-overlay",
        "portage-stable": "/mnt/host/source/src/third_party/portage-stable",
    },
}


def main(argv: List[str]) -> None:
    # Print a marker on every invocation so that unit tests can count how many
    # times this script was run.
    print(
        "[fake_portageq] Command: %s"
        % (" ".join(shlex.quote(s) for s in argv)),
        file=sys.stderr,
    )

    if len(argv) < 3:
        raise Exception("Insufficient number of arguments")

    command = argv[1]
    if command == "get_repos":
        root_dir = argv[2]
        repos = _FAKE_DATA.get(root_dir)
        if not repos:
            raise Exception(f"Unknown root directory: {root_dir}")
        print(" ".join(repos.keys()))
    elif command == "get_repo_path":
        root_dir = argv[2]
        repos = _FAKE_DATA.get(root_dir)
        if not repos:
            raise Exception(f"Unknown root directory: {root_dir}")
        for name in argv[3:]:
            path = repos.get(name)
            if not path:
                raise Exception(f"Unknown repository: {name}")
            print(path)
    else:
        raise Exception(f"Unknown command: {command}")


if __name__ == "__main__":
    main(sys.argv)
