#!/usr/bin/env python3
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""A transparent wrapper of /usr/bin/portageq that caches its results.

It answers to a few subcommands of portageq that don't change across runs for
speed up, assuming that the Portage overlay configuration does not change before
/tmp is cleared.
"""

import logging
import os
import shlex
import subprocess
import sys
import tempfile
from typing import Dict, List, Optional


# Path to the real portageq command.
# Unit tests override the path by setting the environment variable.
_REAL_PORTAGEQ_PATH = os.getenv(
    "PORTAGEQ_WRAPPER_REAL_PORTAGEQ", "/usr/bin/portageq"
)

# Prefix for the cache directories. If it is set to /foo/bar, the cache
# directory for /build/eve will be created at /foo/bar/build/eve.
# Unit tests override the prefix by setting the environment variable.
_CACHE_DIR_PREFIX = os.getenv("PORTAGEQ_WRAPPER_CACHE_DIR_PREFIX", "/")


class _FallbackException(Exception):
    """Causes the wrapper to fallback to the real portageq."""


def _exec_real_portageq(argv: List[str]) -> None:
    """Executes the real portageq command with given arguments.

    Args:
        argv: Arguments passed to the real portageq, including argv[0].

    Returns:
        Never returns on successful execution.
    """
    logging.info(
        "Falling back: %s %s",
        _REAL_PORTAGEQ_PATH,
        " ".join(shlex.quote(s) for s in argv[1:]),
    )
    os.execv(_REAL_PORTAGEQ_PATH, argv)


def _call_real_portageq(args: List[str]) -> bytes:
    """Calls the real portageq command with given arguments.

    It returns the captured standard output. The standard error is passed
    through to the current process' standard error.

    Args:
        args: Arguments passed to the real portageq, excluding argv[0].

    Returns:
        Captured standard output.
    """
    logging.info(
        "Calling: %s %s",
        _REAL_PORTAGEQ_PATH,
        " ".join(shlex.quote(s) for s in args),
    )
    try:
        return subprocess.check_output(
            [_REAL_PORTAGEQ_PATH] + args,
            stdin=subprocess.DEVNULL,
        )
    except subprocess.CalledProcessError:
        raise _FallbackException()


class _PortageqCache:
    """Manages the cache of portageq results."""

    _cache_dir: str

    def __init__(self, root_dir: str):
        cache_root = os.path.join(_CACHE_DIR_PREFIX, root_dir.lstrip("/"))
        if not os.path.isdir(cache_root):
            raise _FallbackException()
        self._cache_dir = os.path.join(
            cache_root, "tmp", "cros-bazel-portageq-cache"
        )
        os.makedirs(self._cache_dir, exist_ok=True)

    def get(self, key: str) -> Optional[bytes]:
        """Retrieves a cached result.

        Args:
            key: A cache key string. It must be valid as a file path component.

        Returns:
            The cached portageq standard output.
        """
        cache_file = os.path.join(self._cache_dir, key)
        try:
            with open(cache_file, "rb") as f:
                return f.read()
        except IOError:
            return None

    def put(self, key: str, value: bytes) -> None:
        """Saves a key/value pair to the cache.

        Args:
            key: A cache key string. It must be valid as a file path component.
            value: A bytes value presenting a portageq standard output.
        """
        cache_file = os.path.join(self._cache_dir, key)
        # pylint: disable=consider-using-with
        temp_file = tempfile.NamedTemporaryFile(
            mode="wb",
            dir=self._cache_dir,
            prefix="." + key + ".",
            delete=False,  # We will call os.rename in the successful case.
        )
        try:
            temp_file.write(value)
            # Replace the file atomically.
            os.rename(temp_file.name, cache_file)
        except Exception:
            os.unlink(temp_file.name)
            raise


def _wrap_get_repos(root_dir: str) -> None:
    cache = _PortageqCache(root_dir)
    key = "get_repos"
    value = cache.get(key)
    if value is None:
        value = _call_real_portageq(["get_repos", root_dir])
        cache.put(key, value)
    sys.stdout.buffer.write(value)


def _wrap_get_repo_path(root_dir: str, names: List[str]) -> None:
    cache = _PortageqCache(root_dir)

    name_to_values: Dict[str, bytes] = {}
    uncached_names: List[str] = []
    uncached_keys: List[str] = []
    for name in names:
        key = f"get_repo_path.{name}"
        value = cache.get(key)
        if value is None:
            uncached_names.append(name)
            uncached_keys.append(key)
        else:
            name_to_values[name] = value

    if uncached_names:
        output = _call_real_portageq(
            ["get_repo_path", root_dir] + uncached_names
        )
        values = output.splitlines(keepends=True)
        assert len(uncached_names) == len(uncached_keys) == len(values)
        for name, key, value in zip(uncached_names, uncached_keys, values):
            name_to_values[name] = value
            cache.put(key, value)

    for name in names:
        sys.stdout.buffer.write(name_to_values[name])


def main(argv: List[str]) -> None:
    logging.basicConfig(
        format="[portageq_wrapper] %(message)s", level=logging.INFO
    )
    logging.info("Command: %s", " ".join(shlex.quote(s) for s in argv))

    try:
        if len(argv) < 3:
            raise _FallbackException()

        command = argv[1]
        if command == "get_repos":
            _wrap_get_repos(argv[2])
        elif command == "get_repo_path":
            _wrap_get_repo_path(argv[2], argv[3:])
        else:
            raise _FallbackException()
    except _FallbackException:
        _exec_real_portageq(argv)


if __name__ == "__main__":
    main(sys.argv)
