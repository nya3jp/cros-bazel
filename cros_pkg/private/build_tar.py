# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""A monkey-patch on top of rules_pkg's build_tar binary.

The original rules strip "/" and "./", while portage tarballs *always* start
with a "./".
"""

from pkg.private.tar import build_tar


orig_normpath = build_tar.normpath


class TarFile(build_tar.TarFile):
    """A TarFile that produces tarballs suitable for metallurgy packages."""

    def normalize_path(self, path: str) -> str:
        normalized = super().normalize_path(path)
        return "./" + normalized


def normpath(path):
    return "./" + orig_normpath(path)


if __name__ == "__main__":
    build_tar.TarFile = TarFile
    build_tar.normpath = normpath
    build_tar.main()
