# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

import dataclasses
import functools
import json
from typing import Dict, List


@dataclasses.dataclass(frozen=True)
class _RawPackageInfo:
    build_deps: List[str]
    runtime_deps: List[str]

    @staticmethod
    def from_json(data: dict) -> '_RawPackageInfo':
        return _RawPackageInfo(
            build_deps=data['buildDeps'],
            runtime_deps=data['runtimeDeps'],
        )


@dataclasses.dataclass(frozen=True)
class PackageInfo(_RawPackageInfo):
    name: str
    transitive_build_deps: List[str]
    transitive_runtime_deps: List[str]


def _transitive_runtime_deps(package: str, raw_info_map: Dict[str, _RawPackageInfo]) -> List[str]:
    deps = set([package])
    stack = [package]
    while stack:
        current = stack.pop()
        info = raw_info_map.get(current)
        for next in info.runtime_deps if info else []:
            if next not in deps:
                deps.add(next)
                stack.append(next)
    return sorted(deps)


def _transitive_build_deps(package: str, raw_info_map: Dict[str, _RawPackageInfo]) -> List[str]:
    info = raw_info_map.get(package)
    direct_build_deps = info.build_deps if info else []
    return sorted(functools.reduce(
        lambda deps, pkg: deps.union(_transitive_runtime_deps(pkg, raw_info_map)),
        direct_build_deps,
        set()))


def load_deps_json(path: str) -> Dict[str, PackageInfo]:
    with open(path) as f:
        data = json.load(f)
    raw_info_map = {
        name: _RawPackageInfo.from_json(value)
        for name, value in data.items()
    }
    full_info_map = {
        name: PackageInfo(
            name=name,
            build_deps=raw_info.build_deps,
            runtime_deps=raw_info.runtime_deps,
            transitive_build_deps=_transitive_build_deps(name, raw_info_map),
            transitive_runtime_deps=_transitive_runtime_deps(name, raw_info_map),
        )
        for name, raw_info in raw_info_map.items()
    }
    return full_info_map
