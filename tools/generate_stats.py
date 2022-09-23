#!/usr/bin/env python3
# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

import csv
import pathlib
import sys
from typing import List

import deps_lib

def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)

def main(argv: List[str]) -> None:
    deps = deps_lib.load_deps_json(argv[1])

    built_packages = set()
    root_dir = pathlib.Path(__file__).parent.parent.parent
    bazel_bin = root_dir / 'bazel-bin'
    for tbz2 in bazel_bin.glob('third_party/*/*/*/*.tbz2'):
        built_packages.add('/'.join(tbz2.parts[-3:-1]))

    good = 0
    bad = 0
    skipped = 0
    status_map = {}
    for package, info in deps.items():
        if package in built_packages:
            status = '✅'
            good = good + 1
        elif all(pkg in built_packages for pkg in info.transitive_build_deps):
            status = '🔥'
            bad = bad + 1
        else:
            status = '⌛'
            skipped = skipped + 1
        for overlay in ('third_party/portage-stable', 'third_party/chromiumos-overlay'):
            overlay_dir = root_dir / overlay / package
            if overlay_dir.exists():
                break
        else:
            raise Exception('%s not found in any overlay' % package)
        label = '//%s/%s' % (overlay, package)
        status_map[label] = status

    csv_out = csv.writer(sys.stdout)
    for label, status in sorted(status_map.items()):
        csv_out.writerow([label, status])

    eprint("Good: %d, Bad: %d, Skipped: %d" % (good, bad, skipped))

if __name__ == '__main__':
    sys.exit(main(sys.argv))
