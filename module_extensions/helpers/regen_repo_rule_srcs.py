#!/usr/bin/env python3
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Writes repo rule dependencies to bzl files.

Performs analysis on a target to determine what code changes could possibly
result in bazel triggering a rebuild of the target, then writes them to .bzl
files
"""

import argparse
import copy
import os
import pathlib
import re
import subprocess


_LABEL_RE = re.compile(
    "^(?:@(?P<repo>[^/]*))?//(?P<pkg>[^:]*)(?::(?P<name>.*))?$"
)
_FILE_CONTENT = """# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# AUTO-GENERATED FILE. DO NOT EDIT.
# To regenerate, run "{regen_cmd}"

{variable} = [
{value}]
"""


class Label:
    """A representation of a bazel label in python"""

    def __init__(self, label: str):
        match = _LABEL_RE.match(label)
        if match is None:
            raise ValueError(f"Invalid absolute label {label}")
        self.repo: str = (match.group("repo") or "").lstrip("@")
        self.pkg: str = match.group("pkg")
        self.name: str = match.group("name") or self.pkg.split("/")[-1]

    @property
    def _key(self):
        return (self.repo, self.pkg, self.name)

    def build_file(self):
        build_file = copy.deepcopy(self)
        build_file.name = "BUILD.bazel"
        return build_file

    def __str__(self):
        return f"@{self.repo or 'cros'}//{self.pkg}:{self.name}"

    def __eq__(self, other):
        return self._key == other._key

    def __lt__(self, other):
        return self._key < other._key

    def __hash__(self):
        return hash(self._key)

    def __repr__(self):
        return f'Label("{self}")'


def calculate_deps(target: Label) -> set[Label]:
    deps = set()
    ps = subprocess.run(
        [
            "bazel",
            "cquery",
            f"kind('source file', deps('{target}'))",
            "--output=starlark",
            "--starlark:expr=target.label",
        ],
        stdout=subprocess.PIPE,
        check=True,
    )

    for label in ps.stdout.decode("utf-8").split():
        label = Label(label)
        # If it's cross-repo, prefer depending on manifests like Cargo.toml/lock
        # rather than the third party code, for example.
        if label.repo == target.repo:
            deps.add(label)
            # The repo rule will depend upon the build file as well as the code.
            deps.add(label.build_file())

    return deps


def output_deps(
    dst: pathlib.Path, variable: str, deps: set[Label], regen_cmd: str
):
    value = "".join([f'    "{label}",\n' for label in sorted(deps)])
    content = _FILE_CONTENT.format(
        variable=variable, value=value, regen_cmd=regen_cmd
    )
    print(f"Writing to {dst.resolve()}")
    dst.write_text(content)


def main():
    workspace = os.environ.get("BUILD_WORKSPACE_DIRECTORY")
    if not workspace:
        raise Exception("BUILD_WORKSPACE_DIRECTORY not found")
    os.chdir(workspace)

    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--target",
        help="The target which you intend to build in a repo rule",
        type=Label,
        required=True,
    )
    parser.add_argument(
        "--output",
        help="The .bzl file to output, relative to the workspace root",
        type=pathlib.Path,
        required=True,
    )
    parser.add_argument(
        "--extra_dep",
        help="Extra dependency that is manually added",
        nargs="*",
        type=Label,
    )
    parser.add_argument(
        "--variable",
        help="The variable name to use in the bzl file",
        default="REPO_RULE_SRCS",
    )
    parser.add_argument(
        "--regen_cmd", help="The command to regenerate the file", required=True
    )
    args = parser.parse_args()

    deps = calculate_deps(args.target)
    # When nargs=0, the value is None instead of [].
    deps.update(args.extra_dep or [])
    output_deps(
        dst=args.output,
        variable=args.variable,
        deps=deps,
        regen_cmd=args.regen_cmd,
    )


if __name__ == "__main__":
    main()
