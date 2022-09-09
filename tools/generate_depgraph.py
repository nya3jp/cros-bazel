#!/usr/bin/env python3
# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

import json
import pathlib
import sys
from typing import List, Optional


def main(argv: List[str]) -> Optional[int]:
    with open(argv[1]) as f:
        package_map = json.load(f)

    built_packages = set()
    bazel_bin = pathlib.Path(__file__).parent.parent.parent / 'bazel-bin'
    for tbz2 in bazel_bin.glob('third_party/*/*/*/*.tbz2'):
        built_packages.add('/'.join(tbz2.parts[-3:-1]))

    print('digraph {')

    for src, info in sorted(package_map.items()):
        build_deps = info['buildDeps']
        runtime_deps = info['runtimeDeps']

        build_only_deps = list(set(build_deps).difference(runtime_deps))

        print('  "%s" [style = filled, fillcolor = %s]' % (src, 'green' if src in built_packages else 'red'))
        if build_only_deps:
            print('  "%s" -> {%s} [style = dashed]' % (src, ' '.join('"%s"' % dst for dst in build_only_deps)))
        if runtime_deps:
            print('  "%s" -> {%s}' % (src, ' '.join('"%s"' % dst for dst in runtime_deps)))

    print('}')


if __name__ == '__main__':
    sys.exit(main(sys.argv))
