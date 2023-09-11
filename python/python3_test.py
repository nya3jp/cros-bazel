# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

import pathlib
import platform
import sys
import unittest


PYTHON_VERSION = "3.11.4"


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
            self.assertIn("/execroot/_main/", path)

    def test_runfiles(self):
        from python.runfiles import runfiles

        r = runfiles.Create()
        path = pathlib.Path(
            r.Rlocation("cros/bazel/python/testdata/example.txt")
        )
        self.assertTrue(path.is_file())


if __name__ == "__main__":
    unittest.main()
