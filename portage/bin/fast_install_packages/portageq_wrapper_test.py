# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Unit tests for portage_wrapper.py."""

import os
import shutil
import subprocess
import sys
import tempfile
from typing import List, Tuple
import unittest


_PORTAGEQ_WRAPPER_PATH = (
    "bazel/portage/bin/fast_install_packages/portageq_wrapper.py"
)
_FAKE_PORTAGEQ_PATH = (
    "bazel/portage/bin/fast_install_packages/testdata/fake_portageq.py"
)


class PortageqTest(unittest.TestCase):
    """Unit tests for portageq_wrapper.py."""

    def setUp(self):
        self._test_root = tempfile.mkdtemp()
        os.makedirs(os.path.join(self._test_root, "build/meow"))

    def tearDown(self):
        self._verify_no_temporary_files()
        shutil.rmtree(self._test_root)

    def _verify_no_temporary_files(self):
        """Verifies that there is no temporary files left in the test root."""
        for dirpath, _, filenames in os.walk(self._test_root):
            for filename in filenames:
                self.assertFalse(
                    filename.startswith("."),
                    "Temporary file left at %s"
                    % os.path.join(dirpath, filename),
                )

    def _run_portageq_wrapper(self, args: List[str]) -> Tuple[str, int]:
        """Runs the portageq wrapper (portageq_wrapper.py) with fakes.

        We set up environment variables so that:
        - The wrapper will call into the fake portageq (fake_portageq.py)
          instead of the system-installed /usr/bin/portageq.
        - The wrapper will create cache files under `self._test_root` instead
          of the host file system.

        Args:
            args: Arguments to pass to the portageq wrapper.

        Returns:
            (stdout, count) where:
            stdout: The standard output from the process.
            count: The number of times the real portageq (fake_portageq.py) was
                executed. We count this by inspecting the standard error.
        """
        env = os.environ.copy()
        env.update(
            {
                "PORTAGEQ_WRAPPER_REAL_PORTAGEQ": _FAKE_PORTAGEQ_PATH,
                "PORTAGEQ_WRAPPER_CACHE_DIR_PREFIX": self._test_root,
            }
        )
        with subprocess.Popen(
            [_PORTAGEQ_WRAPPER_PATH] + args,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            encoding="ascii",
            env=env,
        ) as proc:
            stdout, stderr = proc.communicate()
        sys.stderr.write(stderr)  # Pass-through stderr
        if proc.returncode != 0:
            raise subprocess.CalledProcessError(proc.returncode, proc.args)
        count = stderr.count("[fake_portageq] Command:")
        return stdout, count

    def test_get_repos_cached(self):
        """Verifies that get_repos is cached."""
        self.assertEqual(
            self._run_portageq_wrapper(["get_repos", "/build/meow/"]),
            ("meow chromiumos portage-stable\n", 1),
        )
        self.assertEqual(
            self._run_portageq_wrapper(["get_repos", "/build/meow/"]),
            ("meow chromiumos portage-stable\n", 0),
        )
        self.assertEqual(
            self._run_portageq_wrapper(["get_repos", "/"]),
            ("chromiumos portage-stable\n", 1),
        )
        self.assertEqual(
            self._run_portageq_wrapper(["get_repos", "/"]),
            ("chromiumos portage-stable\n", 0),
        )

    def test_get_repo_path_cached(self):
        """Verifies that get_repo_path is cached."""
        self.assertEqual(
            self._run_portageq_wrapper(
                [
                    "get_repo_path",
                    "/build/meow/",
                    "portage-stable",
                    "chromiumos",
                ]
            ),
            (
                "/mnt/host/source/src/third_party/portage-stable\n"
                "/mnt/host/source/src/third_party/chromiumos-overlay\n",
                1,
            ),
        )
        self.assertEqual(
            self._run_portageq_wrapper(
                ["get_repo_path", "/build/meow/", "chromiumos", "meow"]
            ),
            (
                "/mnt/host/source/src/third_party/chromiumos-overlay\n"
                "/mnt/host/source/src/overlays/overlay-meow\n",
                1,
            ),
        )
        self.assertEqual(
            self._run_portageq_wrapper(
                [
                    "get_repo_path",
                    "/build/meow/",
                    "portage-stable",
                    "chromiumos",
                    "meow",
                ]
            ),
            (
                "/mnt/host/source/src/third_party/portage-stable\n"
                "/mnt/host/source/src/third_party/chromiumos-overlay\n"
                "/mnt/host/source/src/overlays/overlay-meow\n",
                0,
            ),
        )
        self.assertEqual(
            self._run_portageq_wrapper(
                ["get_repo_path", "/", "portage-stable", "chromiumos"]
            ),
            (
                "/mnt/host/source/src/third_party/portage-stable\n"
                "/mnt/host/source/src/third_party/chromiumos-overlay\n",
                1,
            ),
        )
        self.assertEqual(
            self._run_portageq_wrapper(["get_repo_path", "/", "chromiumos"]),
            ("/mnt/host/source/src/third_party/chromiumos-overlay\n", 0),
        )

    def test_get_repo_path_unknown_repos(self):
        """Verifies that get_repo_path fails for unknown repositories."""
        with self.assertRaises(subprocess.CalledProcessError):
            self._run_portageq_wrapper(
                ["get_repo_path", "/build/meow/", "portage-stable", "eve"]
            )
        with self.assertRaises(subprocess.CalledProcessError):
            self._run_portageq_wrapper(
                ["get_repo_path", "/", "portage-stable", "meow"]
            )

    def test_get_repos_invalid_root(self):
        """Verifies that get_repos fails for invalid root directories."""
        with self.assertRaises(subprocess.CalledProcessError):
            self._run_portageq_wrapper(["get_repos", "/build/eve/"])

    def test_get_repo_path_invalid_root(self):
        """Verifies that get_repo_path fails for invalid root directories."""
        with self.assertRaises(subprocess.CalledProcessError):
            self._run_portageq_wrapper(
                ["get_repo_path", "/build/eve/", "portage-stable"]
            )


if __name__ == "__main__":
    unittest.main()
