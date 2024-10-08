# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

import pathlib
import platform
import sys
import unittest


PYTHON_VERSION = "3.11.6"


class Py3Test(unittest.TestCase):
    def test_version(self):
        self.assertEqual(platform.python_version(), PYTHON_VERSION)

    def test_interpreter(self):
        self.assertIn("python3_test.runfiles", sys.executable)

    def test_no_implicit_system_deps(self):
        with self.assertRaises(ImportError):
            import pylint

    def test_no_implicit_directory_deps(self):
        with self.assertRaises(ImportError):
            import pip_test

    def test_hermetic_path(self):
        for path in sys.path:
            # With sandbox_hermetic_tmp enabled, the sandbox runs from inside
            # the /tmp directory
            self.assertTrue(
                path.startswith("/tmp/bazel-working-directory")
                or path.startswith("/tmp/bazel-source-roots"),
                msg=(
                    f"{path} didn't start with /tmp/bazel-working-directory "
                    "or /tmp/bazel-source-roots"
                ),
            )

    def test_runfiles(self):
        from python.runfiles import runfiles

        r = runfiles.Create()
        path = pathlib.Path(
            r.Rlocation("cros/bazel/python/testdata/example.txt")
        )
        self.assertTrue(path.is_file())

    def test_import_from_repo_rule(self):
        # sitecustomize.py should do the repo mapping for us.
        from rules_python.python.runfiles import runfiles


if __name__ == "__main__":
    unittest.main()
