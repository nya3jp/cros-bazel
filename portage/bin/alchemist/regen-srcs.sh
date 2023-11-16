#!/bin/bash
#
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

set -eu -o pipefail

# Workspace root.
cd "$(dirname -- "${BASH_SOURCE[0]}")/../../../.."

ALCHEMIST="bazel/portage/bin/alchemist"

get_src() {
  # Remove entries with spaces because bazel can't handle spaces.
  (cd "$1" && find Cargo.toml src -type f ! -path '*/chromite/.git/*' | grep -v ' ')
}

get_labels() {
  for f in $(get_src "$1"); do
    echo "    \"//$1:${f}\","
  done | LC_ALL=C sort
}

get_list() {
  echo "$1 = ["
  get_labels "$2"
  echo "]"
  echo
}

(
cat <<EOF
# AUTO GENERATED DO NOT EDIT!
# Regenerate this file using ./$(basename -- "$0")
# It should be regenerated each time a file is added or removed.

load(":shared_crates.bzl", "SHARED_CRATES")

EOF
  get_list _DEV_SRCS_NO_LOCK "${ALCHEMIST}"

  echo "_SHARED_CRATE_FILES = ["
  grep -oP '(?<=^    "//).*(?=:srcs",$)' "${ALCHEMIST}/shared_crates.bzl" | \
  while read -r crate; do
    get_labels "${crate}"
  done
  echo "]"

  cat <<EOF

_LOCK = "//bazel/portage/bin/alchemist:Cargo.lock"
_DEV_SRCS = _DEV_SRCS_NO_LOCK + [_LOCK]

_RELEASE_SRCS = [src for src in _DEV_SRCS if "/testdata/" not in src]

ALCHEMIST_BAZEL_LIB_SRCS = [
    Label(x)
    for x in _DEV_SRCS + SHARED_CRATES
    if not x.startswith("//bazel/portage/bin/alchemist:src/bin/alchemist")
]

ALCHEMIST_BAZEL_BIN_SRCS = [
    Label(x.replace(":src/bin/alchemist/", "/src/bin/alchemist:"))
    for x in _DEV_SRCS + SHARED_CRATES
    if x.startswith("//bazel/portage/bin/alchemist:src/bin/alchemist")
]

ALCHEMIST_REPO_RULE_SRCS = [
    Label(x) for x in _RELEASE_SRCS + _SHARED_CRATE_FILES
]
EOF

) > bazel/portage/bin/alchemist/src.bzl

bazel run "//bazel/portage/bin/alchemist/src/bin/alchemist:regen_repo_rule_srcs"

echo Done
