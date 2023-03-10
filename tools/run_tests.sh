#!/bin/bash -eu

# Copyright 2023 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

echo "Running precommits" >&2

export BOARD=${BOARD:-amd64-generic}

TARGETS=(
  //rules_cros/third_party/...
  //rules_cros/toolchains/...
  //bazel/ebuild/...
   # Check that build_package works.
  @portage//dev-libs/leveldb
)

# Optimization: don't bust the cache between the bazel test and bazel build.
export CACHE_BUST_DATE="${CACHE_BUST_DATE:-$(date --iso-8601=ns)}"

set -x

# We can run cargo in parallel with bazel.
cargo test --package alchemist -- --nocapture &

# Despite the name, bazel test also builds non-test targets if they're listed.
bazel test \
  --test_size_filters=small \
  --config=format \
  --keep_going \
  -- \
  "${TARGETS[@]}"

wait
