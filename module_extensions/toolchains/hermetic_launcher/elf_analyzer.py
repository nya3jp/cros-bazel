# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Analyzes elf files to create dependency metadata.

By only including the files that a binary actually depends on, we can reduce
the size of the dependencies from ~300MB to ~4MB.
"""

import dataclasses
import itertools
import os
from typing import Optional

from elftools.elf.dynamic import DynamicSection
from elftools.elf.elffile import ELFFile


@dataclasses.dataclass
class ElfMetadata:
    """Metadata about an elf file."""

    soname: Optional[str]
    deps: set[str]


def get_elf_metadata(path: str) -> ElfMetadata:
    """Calculates metadata for an elf file.

    Args:
        path: The path to the elf file

    Returns:
        The relevant metadata for the elf file.
    """
    soname = None
    deps = set()
    with open(path, "rb") as f:
        for sect in ELFFile(f).iter_sections():
            if isinstance(sect, DynamicSection):
                for tag in sect.iter_tags():
                    if hasattr(tag, "needed"):
                        deps.add(tag.needed)
                    if hasattr(tag, "soname"):
                        soname = tag.soname

    # We don't care about the dependency on the system interpreter.
    deps.discard("ld-linux-x86-64.so.2")

    return ElfMetadata(soname=soname, deps=deps)


def _transitive(dep_map: dict[str, set[str]], value: str) -> set[str]:
    """Calculates the transitive dependencies of a single entry in a dep map.

    Args:
        dep_map: A mapping from soname to the direct dependencies of that
          shared library.
        value: A shared library name corresponding to a kep in dep_map.

    Returns:
        The transitive dependencies of that value in the dependency map.
    """
    out = set(dep_map[value])
    for entry in dep_map[value]:
        out.update(_transitive(dep_map, entry))
    return out


def get_dependency_map(paths: list[str]) -> dict[str, list[str]]:
    """Calculates the dependencies of a given set of elf files.

    Given a list of libraries, returns a mapping from soname to a list of
    files required to load that library.

    Args:
        paths: A list of shared libraries available.

    Returns:
        A mapping from soname to a list of files required to load.
    """
    soname_to_filename = {}
    out = {}
    for path in paths:
        metadata = get_elf_metadata(path)
        soname = metadata.soname or os.path.basename(path)
        soname_to_filename[soname] = os.path.basename(path)
        out[soname] = metadata.deps

    for soname, deps in sorted(out.items()):
        deps.update(itertools.chain(*[_transitive(out, dep) for dep in deps]))

    # If a binary references libfoo.so, then it will need the transitive
    # dependencies of libfoo.so, but also libfoo.so itself.
    for soname, deps in out.items():
        deps.add(soname)

    return {
        soname: sorted(soname_to_filename[dep] for dep in deps)
        for soname, deps in out.items()
    }
