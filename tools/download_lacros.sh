#!/bin/bash -eu
# Copyright 2023 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# Downloading distfiles of chromeos-base/chromeos-lacros via anonymous https://
# fails for 403 errors, likely because we're pinned to an old ChromeOS snapshot.
# This script works around the problem by downloading them via authenticated
# gs:// and saving to Bazel's content addressable storage directly.

readonly cache_base_dir="${HOME}/.cache/bazel/_bazel_${USER}/cache/repos/v1/content_addressable/sha512"
readonly specs=(
  # <URL>=<SHA512>
  "gs://chrome-unsigned/desktop-5c0tCh/107.0.5257.0/lacros-arm64/metadata.json=28ff38f45369a5cfcc198fb5f64f171d2f48ed0b2580d8bbe0474a622f60a03288335b145f0528098284b0f808faf96853d5c3b6b50ea04368e58a59d3bfc536"
  "gs://chrome-unsigned/desktop-5c0tCh/107.0.5257.0/lacros-arm64/lacros_compressed.squash=bf9b1df731d21f3b667b189409f20010cfda3d76096de04002fb87de839d3321bd7d5589361430d8e5fc062f448968c9bd5eec48ef97f34c22826bac33ec50f6"
  "gs://chrome-unsigned/desktop-5c0tCh/107.0.5257.0/lacros64/metadata.json=28ff38f45369a5cfcc198fb5f64f171d2f48ed0b2580d8bbe0474a622f60a03288335b145f0528098284b0f808faf96853d5c3b6b50ea04368e58a59d3bfc536"
  "gs://chrome-unsigned/desktop-5c0tCh/107.0.5257.0/lacros64/lacros_compressed.squash=1fedc1199e60df3ec9f517d9c719656eeefc1468122abca82c638bbbb1ea6160e920c3509c15b2d21533178521aaabd81894ef73a75be0e8fcfb1b4ad75a4e1a"
)

for spec in "${specs[@]}"; do
  IFS='=' read -r url hash <<< "${spec}"
  cache_dir="${cache_base_dir}/${hash}"
  if [[ -f "${cache_dir}/file" ]]; then
    continue
  fi

  mkdir -p "${cache_dir}"
  gcloud storage cp "${url}" "${cache_dir}/file"
done
