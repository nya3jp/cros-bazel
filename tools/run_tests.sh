#!/bin/bash -eu

# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# Usage: [SKIP_CARGO/BAZEL/PORTAGE_TESTS=1] run_tests.sh [bazel test args]
# eg. SKIP_CARGO_TESTS=1 run_tests.sh --config=hermetic_toolchains

echo "Running precommits" >&2

export BOARD="${BOARD:-amd64-generic}"

TARGETS=(
  //rules_cros/third_party/...
  //rules_cros/toolchains/...
  //bazel/...
  -//bazel/images/...
  -//bazel/prebuilts/...
  -//bazel/rules_cros/...
)

if [[ -z "${SKIP_PORTAGE_TESTS:=}" ]]; then
  # Check that build_package works.
  TARGETS+=( @portage//dev-libs/leveldb )
fi

set -x

if [[ -z "${SKIP_CARGO_TESTS:=}" ]]; then
  # We can run cargo in parallel with bazel.
  cargo test --package alchemist -- --nocapture &
fi

if [[ -z "${SKIP_BAZEL_TESTS:=}" ]]; then
  # Despite the name, bazel test also builds non-test targets if they're listed.
  bazel test \
    --test_size_filters=small \
    --config=format \
    --keep_going \
    "$@" \
    -- \
    "${TARGETS[@]}"
fi

if [[ -z "${SKIP_CARGO_TESTS:=}" ]]; then
  wait
fi
