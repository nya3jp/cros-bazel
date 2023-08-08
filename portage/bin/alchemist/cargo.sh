#!/bin/bash

# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

set -eu -o pipefail

LOCKFILE=$(rlocation cros/bazel/portage/bin/alchemist/Cargo.lock)
SRCS=$(dirname "${LOCKFILE}")
CARGO=$(rlocation rust_host_tools/bin/cargo)
RUSTC=$(rlocation rust_host_tools/bin/rustc)
RUSTDOC=$(rlocation rust_host_tools/bin/rustdoc)

WORKSPACE_SRCS=$(dirname "$(realpath "${LOCKFILE}")")

# If BUILD.bazel exists, we're not in RBE, since BUILD.bazel is explicitly
# excluded from the glob.
if [[ -f "${WORKSPACE_SRCS}/BUILD.bazel" ]]; then
  # Ideally we'd set BUILD_CACHE to ${WORKSPACE_SRCS}/target. However, this
  # doesn't work because when running from a bazel test, the bazel sandbox
  # remounts the workspace root as readonly.
  BUILD_CACHE="/tmp/alchemist_build_cache${WORKSPACE_SRCS}"
  mkdir -p "${BUILD_CACHE}"

  # This allows cargo to actually use the build cache.
  # If we don't do this and run the test multiple times, you're running:
  # "cd execroot_1 && CARGO_TARGET_DIR=foo cargo test"
  # "cd execroot_2 && CARGO_TARGET_DIR=foo cargo test"
  # And then even if they use the same build cache directory, it won't get a
  # cache hit because the source directory was different.
  cd "${WORKSPACE_SRCS}"
elif [[ -n "${TEST_TMPDIR:=}" ]]; then
  mkdir "${TEST_TMPDIR}/target"
  BUILD_CACHE="${TEST_TMPDIR}/target"
  cd "${SRCS}"
else
  echo "Bazel needs a directory for the build cache for cargo" >&2
  exit 1
fi

RUSTC="${RUSTC}" \
  RUSTDOC="${RUSTDOC}" \
  CARGO_TARGET_DIR="${BUILD_CACHE}" \
  exec "${CARGO}" "$@"
