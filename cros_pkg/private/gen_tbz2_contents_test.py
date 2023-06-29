# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

import pathlib
import tarfile
import unittest

from python.runfiles import runfiles


class TarballTest(unittest.TestCase):
    def test_tarball(self):
        r = runfiles.Create()
        path = pathlib.Path(
            r.Rlocation("cros/bazel/cros_pkg/private/direct_example.tbz2")
        )
        with tarfile.open(path) as tar:
            members = {member.name: member for member in tar.getmembers()}
            files = sorted(members)
            self.assertIn("pkg/empty_dir", files)
            self.assertIn("pkg/file", files)
            self.assertIn("pkg/filegroup_file", files)
            self.assertIn("pkg/link", files)

            f = members["pkg/file"]
            # This should be hardcoded by rules_pkg to prevent caching issues.
            self.assertEqual(f.mtime, 946684800)
            self.assertEqual(f.mode, 0o644)
            self.assertEqual(f.uname, "")
            self.assertEqual(f.gname, "")

            self.assertIn("usr/share/doc/doc1.md", files)
            self.assertIn("usr/share/doc/doc2.md", files)

            self.assertIn("usr/bin/renamed_bin", files)
            bin = members["usr/bin/renamed_bin"]
            self.assertEqual(bin.mode, 0o755)
            self.assertEqual(bin.uname, "root")
            self.assertEqual(bin.gname, "root")
            self.assertEqual(bin.uid, 0)
            self.assertEqual(bin.gid, 0)

            self.assertIn("path/to/file1", files)
            self.assertIn("path/to/file2", files)
            file1 = members["path/to/file1"]
            self.assertEqual(file1.mode, 0o640)

            self.assertIn("demo/files_only", files)
            self.assertIn("demo/testdata/strip_prefix/from_current_pkg", files)
            self.assertIn("demo/strip_prefix/from_pkg", files)

            self.assertIn("tmp/dest", files)


if __name__ == "__main__":
    unittest.main()
