#!/usr/bin/env python3
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""A tool to regenerate symlinks.

We use repo manifest's feature to generate symlinks, but repo has bugs where it
fails to create/delete symlinks when needed. This script helps you workaround
the issue until the upstream fixes it.
"""

import contextlib
import dataclasses
import os
import pathlib
import subprocess
from typing import List
from xml.etree import ElementTree


def _find_root_dir() -> pathlib.Path:
    """Finds the root directory of the ChromeOS source checkout."""
    current = pathlib.Path(__file__).parent
    while current != "/":
        if current.joinpath(".repo").is_dir():
            return current
        current = current.parent
    raise Exception("ChromeOS source root directory not found")


@dataclasses.dataclass(frozen=True)
class LinkFile:
    """A plan to create a symlink."""

    location: pathlib.Path
    target: pathlib.PurePath


def _parse_manifest(root_dir: pathlib.Path) -> List[LinkFile]:
    tree = ElementTree.parse(root_dir.joinpath(".repo/manifests/_bazel.xml"))
    root = tree.getroot()

    links: List[LinkFile] = []

    for project in root.findall("./project"):
        for linkfile in project.findall("./linkfile"):
            location = root_dir.joinpath(linkfile.attrib["dest"])
            target_abs = root_dir.joinpath(
                project.attrib["path"], linkfile.attrib["src"]
            )
            target = os.path.relpath(target_abs, location.parent)
            links.append(
                LinkFile(
                    location=location,
                    target=pathlib.PurePath(target),
                )
            )

    return links


def _is_git_tracked(path: pathlib.Path) -> bool:
    exit_code = subprocess.call(
        ["git", "ls-files", "--error-unmatch", path.name],
        cwd=path.parent,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    return exit_code == 0


def main():
    """The entry point of the program."""
    root_dir = _find_root_dir()
    links = _parse_manifest(root_dir)

    print("Searching for existing symlinks...")

    # Limit the search to BUILD.bazel only, because there might be random
    # untracked symlinks the user created for development. This can fail to
    # remove some stale symlinks, but until today stale symlinks were mostly
    # named BUILD.bazel.
    for path in root_dir.glob("**/BUILD.bazel"):
        if not path.is_symlink():
            continue
        if _is_git_tracked(path):
            continue
        print("Removing %s" % path)
        path.unlink()

    # Create symlinks. We process all symlinks, not only BUILD.bazel.
    for link in links:
        print("Creating %s -> %s" % (link.location, link.target))
        with contextlib.suppress(FileNotFoundError):
            link.location.unlink()
        link.location.symlink_to(link.target)


if __name__ == "__main__":
    main()
