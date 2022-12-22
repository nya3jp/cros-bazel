#!/usr/bin/env python3

"""
A bazelified cargo init. Calls cargo init under the hood, then sets up your
build files as required to make it a bazel package.
"""

import argparse
import os
import pathlib
import subprocess

_BUILDOZER_COMMAND_FAILED = 2
_BUILDOZER_NO_CHANGES = 3

_UPDATE_CRATES_BUILD_FILE = pathlib.Path('bazel/toolchains/rust/local_crates.bzl')

parser = argparse.ArgumentParser()
parser.add_argument('target')
parser.add_argument('-b', '--bin', action='store_true')
parser.add_argument('-l', '--lib', action='store_true')
parser.add_argument('-e', '--existing', action='store_true')


def run(cmd_args, *args, **kwargs):
  print('Running', ' '.join(f"'{arg}'" for arg in cmd_args))
  return subprocess.run(cmd_args, *args, **kwargs, check=True)


def buildozer(cmd: str, target: str, allow_failures=False):
  try:
    run(['buildozer', cmd, target])
  except subprocess.CalledProcessError as e:
    if e.returncode != _BUILDOZER_NO_CHANGES and not (allow_failures and e.returncode == _BUILDOZER_COMMAND_FAILED):
      raise


def add_line_between_sorted(path: pathlib.Path, before: str, after: str, line: str, key = None):
  """Adds a line between before and after in the specified file.

  Ensures all the entries in the list are unique and sorted.
  """
  contents = path.read_text().split('\n')
  start = contents.index(before) + 1
  end = start + contents[start:].index(after)
  # Use a set to dedup them, to ensure that new_crate is idempotent.
  lines = contents[:start] + sorted(set(contents[start:end] + [line]),
                                          key=key) + contents[end:]
  path.write_text('\n'.join(lines))


def main():
  args = parser.parse_args()
  target = args.target

  if args.bin == args.lib:
    raise parser.error('Must provide exactly one of --bin and --lib')

  workspace = pathlib.Path(os.environ['BUILD_WORKSPACE_DIRECTORY'])
  os.chdir(workspace)

  if not target.startswith('//'):
    raise ValueError('The target must be an absolute bazel target (eg. //path/to/crate)')

  if ':' in target:
    package, label = target.rsplit(':', 1)
  else:
    package = target
    label = target.split('/')[-1]

  directory = pathlib.Path(package[2:])
  if not args.existing:
    run(['cargo', 'init', '--bin' if args.bin else '--lib', str(directory)])

  # Buildozer can do this, but it can't add it in sorted order.
  add_line_between_sorted(_UPDATE_CRATES_BUILD_FILE, 'LOCAL_CRATES = {', '}',
                          f'    "{directory.name}": "{target}",')

  print("Adding a rule for the crate")
  rule = 'rust_binary_crate' if args.bin else 'rust_library_crate'
  buildozer(f'new_load //bazel/toolchains/rust:defs.bzl {rule}', f'{package}:__pkg__')
  # Allow_failures here and below to ensure that running this script to update an
  # existing rust package works.
  buildozer(f'new {rule} {label}', f'{package}:__pkg__', allow_failures=True)

  print("Adding a test rule")
  buildozer(f'new_load //bazel/toolchains/rust:defs.bzl rust_crate_test', f'{package}:__pkg__')
  buildozer(f'new rust_crate_test {label}_test', f'{package}:__pkg__', allow_failures=True)
  buildozer(f'set crate ":{label}"', f'{package}:{label}_test')

  run(['bazel', 'run', '//bazel/toolchains/rust:update_crates'])

if __name__ == '__main__':
  main()