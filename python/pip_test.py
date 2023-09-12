# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

import unittest


class PipTest(unittest.TestCase):
    def test_deps(self):
        import pylint

    def test_third_party_deps(self):
        import pylint as normal_pylint

        from third_party import pylint as third_party_pylint

        self.assertIs(normal_pylint, third_party_pylint)


if __name__ == "__main__":
    unittest.main()
