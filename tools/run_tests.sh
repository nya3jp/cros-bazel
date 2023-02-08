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

# Runs tests first to ensure that, for example, if mountsdk is broken, we don't
# get a failure like "failed to build ebuild", but instead fail :mountsdk_test.
bazel test --test_size_filters=small -- "${TARGETS[@]}"
bazel build -- "${TARGETS[@]}"

wait
