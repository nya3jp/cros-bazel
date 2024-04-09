#!/usr/bin/env python3
# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Mirrors any content that is not yet mirrored.

Attempts to build targets in strict mode, and parses stderr to determine which
files are missing from the mirror, mirroring them as required.
"""

import argparse
import asyncio
import concurrent.futures
import getpass
import logging
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile
from typing import List, Optional, Tuple
import urllib
import urllib.request


_DOWNLOAD_CACHE = "/tmp/cros_mirror_repo_cache"
# pylint: disable=line-too-long
_PREFIX = "https://commondatastorage.googleapis.com/chromeos-localmirror/cros-bazel/mirror/"
_DOWNLOAD_ERROR = re.compile(
    f"Error downloading \\[{_PREFIX}(.*?)\\] to .*: GET returned 404 Not Found$"
)


if sys.version_info < (3, 11):
    ExceptionGroup = lambda _, exceptions: Exception(exceptions)


class MirrorError(Exception):
    """Errors during mirroring of content"""


def _mirror(uri: str) -> Tuple[str, Optional[MirrorError]]:
    src = f"https://{uri}"
    dst = f"gs://chromeos-localmirror/cros-bazel/mirror/{uri}"
    try:
        with tempfile.TemporaryDirectory() as temp_dir:
            local = pathlib.Path(temp_dir, "file")
            try:
                urllib.request.urlretrieve(src, local)
            except BaseException as e:
                raise MirrorError(f"Unable to download from {src}") from e
            try:
                subprocess.run(
                    [
                        "gsutil",
                        "cp",
                        "-n",
                        "-a",
                        "public-read",
                        str(local),
                        dst,
                    ],
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                    check=True,
                )
            except BaseException as e:
                raise MirrorError(f"Unable to upload to {dst}") from e
    except MirrorError as e:
        return uri, e
    return uri, None


async def _mirror_missing(args: List[str]) -> int:
    logging.info("Running %s", " ".join(args))
    ps = await asyncio.create_subprocess_exec(
        *args,
        stdout=asyncio.subprocess.DEVNULL,
        stderr=asyncio.subprocess.PIPE,
    )

    futures = []
    requests = set()

    with concurrent.futures.ThreadPoolExecutor() as executor:
        while True:
            line = await ps.stderr.readline()
            if not line:
                break
            line = line.decode("utf-8").rstrip()
            print(line, file=sys.stderr)

            download_err = _DOWNLOAD_ERROR.search(line)
            if download_err is not None:
                uri = download_err.group(1)
                if uri not in requests:
                    futures.append(executor.submit(_mirror, uri))
                    requests.add(uri)

        # Wait for bazel to complete before reporting any errors for mirroring.
        # This ensures we can't get interspersed mirroring logs and bazel logs.
        returncode = await ps.wait()

        errors = []
        for fut in concurrent.futures.as_completed(futures):
            uri, err = fut.result()
            if err is None:
                logging.info("Mirroring succeeded: %s", uri)
            else:
                errors.append(err)

    if len(errors) == 1:
        raise errors[0]
    elif errors:
        raise ExceptionGroup(f"Failed to mirror {len(errors)} files", errors)
    elif returncode and not futures:
        logging.critical("bazel failed for a reason unrelated to mirroring")
        sys.exit(1)
    return len(futures)


def main(
    args: List[str],
    expunge: bool,
    clear_download_cache: bool,
):
    output_base = os.path.expanduser(
        f"~/.cache/bazel/_bazel_{getpass.getuser()}/force_mirror"
    )
    common_args = [
        "bazel",
        f"--output_base={output_base}",
    ]
    if expunge:
        logging.info("Running bazel clean --expunge")
        subprocess.run([*common_args, "clean", "--expunge"], check=True)
        logging.info("bazel output directory is cleaned")
    if clear_download_cache:
        logging.info("Clearing bazel download cache")
        shutil.rmtree(_DOWNLOAD_CACHE)

    bazel_args = [
        *common_args,
        "build",
        # This way we can collect multiple download errors at once.
        "--keep_going",
        "--noremote_upload_local_results",
        # Error out if we haven't successfully downloaded.
        "--config=strict_mirror",
        # If we use the regular repository cache, this won't work properly.
        f"--repository_cache={_DOWNLOAD_CACHE}",
        # We don't actually need to build to verify that we mirrored
        # successfully.
        "--nobuild",
        *args,
    ]

    loop = asyncio.get_event_loop()
    while True:
        n_mirrored = loop.run_until_complete(_mirror_missing(bazel_args))
        if n_mirrored == 0:
            break
        logging.info("Mirrored %d urls", n_mirrored)


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)

    parser = argparse.ArgumentParser()

    parser.add_argument(
        "--expunge",
        action="store_true",
        help="Runs 'bazel clean --expunge' before attempting to mirror",
    )

    parser.add_argument(
        "--clear_download_cache",
        action="store_true",
        help="Deletes the download cache before attempting to mirror",
    )

    parsed, leftover = parser.parse_known_args()
    main(args=leftover, **vars(parsed))
