# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""A test for the symlink bazel rule."""

import pathlib
import unittest

from python.runfiles import runfiles


class SymlinkTest(unittest.TestCase):
    """Tests for the symlink bazel rule."""

    def test_symlink(self):
        r = runfiles.Create()
        path = pathlib.Path(r.Rlocation("files/dumb_init"))

        self.assertTrue(path.is_symlink())
        resolved = path.resolve(strict=False)
        self.assertTrue(str(resolved).endswith("dumb_init/file/downloaded"))
        self.assertTrue(resolved.is_file())


if __name__ == "__main__":
    unittest.main()
