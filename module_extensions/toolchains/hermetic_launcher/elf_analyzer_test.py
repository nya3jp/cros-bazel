# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Tests for the elf analyzer."""

import os
import unittest

from rules_python.python.runfiles import runfiles

from cros.bazel.module_extensions.toolchains.hermetic_launcher import (
    elf_analyzer,
)


class ElfAnalyzerTest(unittest.TestCase):
    """Tests for elf analyzer."""

    def test_direct_deps(self):
        path = runfiles.Create().Rlocation("toolchain_sdk/lib/libc++.so.1")

        metadata = elf_analyzer.get_elf_metadata(path)

        self.assertEqual(metadata.soname, "libc++.so.1")
        self.assertEqual(
            metadata.deps, {"libc.so.6", "libc++abi.so.1", "libgcc_s.so.1"}
        )

    def test_transitive_libs(self):
        lib_dir = runfiles.Create().Rlocation("toolchain_sdk/lib")
        libs = [os.path.join(lib_dir, f) for f in os.listdir(lib_dir)]

        dep_map = elf_analyzer.get_dependency_map(libs)
        self.assertEqual(
            dep_map["libc++.so.1"],
            [
                "libc++.so.1",
                "libc++abi.so.1",
                "libc.so.6",
                "libgcc_s.so.1",
                "libm.so.6",
            ],
        )


if __name__ == "__main__":
    unittest.main()
