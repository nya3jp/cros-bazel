# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""This file runs before any python binary.

It is required to fix various incompatibilities with imports in python.
"""

import collections
import importlib.util
import os
import sys
import types

from python.runfiles import runfiles


r = runfiles.Create()
runfiles_root = r._python_runfiles_root


def should_override_import(name) -> bool:
    spec = importlib.util.find_spec(name)
    if spec is None:
        return True
    dirs = spec.submodule_search_locations
    if len(dirs) != 1:
        return False
    # The repo is unmapped. However, we re-map it anyway in case it's overridden
    # by another repo mapping.
    if dirs[0] == os.path.join(runfiles_root, name):
        return True

    raise NotImplementedError(
        f"{target_name} is already a library that can be imported, but you "
        f"defined a repo rule @{target_name}, which clashes with that. "
        "Please inform msta@ so he can implement this."
    )


def valid_python_module(name):
    # The __main__ module is reserved.
    if name == "__main__":
        return False
    # python doesn't allow you to import with dashes.
    if "-" in name:
        return False
    return True


class ThirdParty(types.ModuleType):
    """This rewrites 'from third_party import bar' to 'import bar'"""

    __file__ = "Fake third party module for bazel"
    __path__ = None

    def __getattr__(self, item):
        return __import__(item)


sys.modules["third_party"] = ThirdParty(name="third_party")


class RepoMappedModule(types.ModuleType):
    """A virtual module corresponding to @foo.

    Due to repo mapping, one repo might see @pylint -> @pip~pylint~1.2.3, while
    another might see @pylint -> @pip~pylint~2.3.4.

    This module just dispatches it to the appropriate directory based on the
    requester.
    """

    def __init__(self, name: str, repo_mapping: dict[str, str]):
        super().__init__(name=name)
        self.__file__ = f"repo rule @{name}"
        self._repo_mapping = repo_mapping
        self._module_mapping: dict[str, types.ModuleType] = {}

    def __getattr__(self, item):
        """Dispatches the getattr to the real module."""
        current_real = r.CurrentRepository(frame=2)
        target_real = self._repo_mapping[current_real]
        mod = self._module_mapping.get(target_real, None)
        if mod is None:
            spec = importlib.machinery.ModuleSpec(
                name=self.__name__,
                loader=None,
            )
            spec.submodule_search_locations = [
                os.path.join(runfiles_root, target_real)
            ]
            mod = importlib.util.module_from_spec(spec)
            self._module_mapping[target_real] = mod

        return getattr(mod, item)


global_repo_mapping = collections.defaultdict(dict)
repo_mapped_modules = {}

for (current_real, target_name), target_real in r._repo_mapping.items():
    global_repo_mapping[target_name][current_real] = target_real


for target_name, repo_mapping in global_repo_mapping.items():
    if valid_python_module(target_name):
        # Ensure that if we create a repo called foo, we don't override the
        # existing foo in the standard library.
        if should_override_import(target_name):
            mod = RepoMappedModule(name=target_name, repo_mapping=repo_mapping)
            repo_mapped_modules[target_name] = mod
            sys.modules[target_name] = mod
