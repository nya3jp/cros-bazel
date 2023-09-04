#!/bin/bash -eu

# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# A wrapper for cargo to be used with cargo_bazel_bootstrap to get incremental
# builds for repo rules.

# This overrides cargo, so we don't get told the real cargo. Work it out for
# ourselves.
RUSTC="${RUSTC:-}"
if [[ -z  "${RUSTC}" ]]; then
  echo "RUSTC must be set to use the alchemist cargo wrapper" >&2
  exit 1
fi
CARGO="$(dirname "${RUSTC}")/cargo"
ARGS=("$@")

# Retrieves the index for the value of an arg.
# eg. If ARGS is [cargo, build, --foo, bar], then get_index --foo would return
# 3, since ARGS[3] = bar (the value of --foo).
get_index() {
  field="$1"
  for i in $(seq 0 $(("${#ARGS[@]}" - 1))); do
    if [[ "${ARGS[${i}]}" = "${field}" ]]; then
      echo "$(("${i}" + 1))"
    fi
  done
}

# The target directory, by default, is set to be contained within the repo rule.
# This means that we can't do incremental builds. We solve this problem by just
# setting the target directory to the same one rust always uses, which is
# $(dirname Cargo.toml)/target.
manifest_path="${ARGS[$(get_index "--manifest-path")]}"
new_target_dir="$(dirname "${manifest_path}")/target"

target_dir_index=$(get_index "--target-dir")
old_target_dir="${ARGS[${target_dir_index}]}"

ARGS["${target_dir_index}"]="${new_target_dir}"

set +e
"${CARGO}" "${ARGS[@]}"
EXIT_STATUS="$?"

# The repo rule reads profile/alchemist (eg. release/alchemist).
# But it's expecting it to be within the old target directory.
for f in "${new_target_dir}"/*; do
  name="$(basename "${f}")"
  rm -f "${old_target_dir}/${name}"
  ln -s "${f}" "${old_target_dir}/${name}"
done


exit "${EXIT_STATUS}"
