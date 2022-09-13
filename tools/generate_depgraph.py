#!/usr/bin/env python3
# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

import pathlib
import sys
from typing import List, Optional

import deps_lib


def main(argv: List[str]) -> Optional[int]:
    deps = deps_lib.load_deps_json(argv[1])

    built_packages = set()
    bazel_bin = pathlib.Path(__file__).parent.parent.parent / 'bazel-bin'
    for tbz2 in bazel_bin.glob('third_party/*/*/*/*.tbz2'):
        built_packages.add('/'.join(tbz2.parts[-3:-1]))

    print('digraph {')

    for package, info in sorted(deps.items()):
        build_only_deps = list(set(info.build_deps).difference(info.runtime_deps))

        if package in built_packages:
            color = 'green'
        elif all(pkg in built_packages for pkg in info.transitive_build_deps):
            color = 'red'
        else:
            color = 'yellow'

        print('  "%s" [style = filled, fillcolor = %s]' % (package, color))
        if build_only_deps:
            print('  "%s" -> {%s} [style = dashed]' % (package, ' '.join('"%s"' % dst for dst in build_only_deps)))
        if info.runtime_deps:
            print('  "%s" -> {%s}' % (package, ' '.join('"%s"' % dst for dst in info.runtime_deps)))

    print('}')


if __name__ == '__main__':
    sys.exit(main(sys.argv))
