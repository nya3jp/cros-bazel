#!/bin/bash -eu

# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# Usage: [SKIP_PORTAGE_TESTS=1] run_tests.sh [bazel test args]
# eg. SKIP_PORTAGE_TESTS=1 run_tests.sh --config=hermetic_toolchains

echo "Running precommits" >&2

export BOARD="${BOARD:-amd64-generic}"

# cd to the WORKSPACE_ROOT (src/)
cd "$(dirname "${BASH_SOURCE[0]}")/../../.."

TARGETS=(
  //bazel/...
  -//bazel/images/...
  -//bazel/portage/repo_defs/prebuilts/...
  @prebuilt_sdk_demo//...
)

if [[ -z "${SKIP_PORTAGE_TESTS:=}" ]]; then
  # Check that build_package works.
  TARGETS+=( @portage//dev-libs/leveldb )
fi

set -x

  # Despite the name, bazel test also builds non-test targets if they're listed.
exec bazel test \
    --test_size_filters=small \
    --config=format \
    --keep_going \
    "$@" \
    -- \
    "${TARGETS[@]}"
