# Copyright 2023 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

import sys
import pathlib
import platform
import unittest

PYTHON_VERSION = "3.10.8"


class Py3Test(unittest.TestCase):

    def test_version(self):
        self.assertEqual(platform.python_version(), PYTHON_VERSION)

    def test_interpreter(self):
        self.assertIn('python3_test.runfiles', sys.executable)

    def test_no_implicit_deps(self):
        with self.assertRaises(ImportError):
            import pylint

    def test_runfiles(self):
        from python.runfiles import runfiles
        r = runfiles.Create()
        path = pathlib.Path(r.Rlocation("cros/rules_cros/toolchains/testdata/example.txt"))
        self.assertTrue(path.is_file())


if __name__ == "__main__":
    unittest.main()
