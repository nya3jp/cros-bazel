#!/bin/bash

# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

STAT_FMT='%n mode:%a uid:%u user:%U gid:%g group:%G'

# Chown doesn't work unless you're root.
if [[ $(id -u) -ne 0 ]]; then
  exec unshare --map-user=0 --map-group=0 "${BASH_SOURCE[0]}" "$@"
fi

set -eu -o pipefail

deploy=$(rlocation cros/bazel/cros_pkg/examples/packaging/packaging_deploy_local)
tarball=$(rlocation cros/bazel/cros_pkg/private/direct_example.tbz2)

tmp=$(mktemp -d)

# deletes the temp directory
function cleanup {
  rm -rf "${tmp}"
}

# register the cleanup function to be called on the EXIT signal
trap cleanup EXIT

got_dir="${tmp}/got"
want_dir="${tmp}/want"
mkdir "${got_dir}" "${want_dir}"

# Extracting tbz2 files with the tar command doesn't work because the zstd
# decoder complains about the trailing bytes.
tar -xf "${tarball}" -C "${want_dir}" 2>/dev/null || true
"${deploy}" "--install-dir=${got_dir}"

diff --recursive --no-dereference "${got_dir}" "${want_dir}"

get_permissions() {
  (cd "$1" && find . | sort | tail +2 | xargs stat -c "${STAT_FMT}") > "$2"
}

# Diff doesn't compare permissions / owners.
get_permissions "${got_dir}" "${got_dir}_permissions"
get_permissions "${want_dir}" "${want_dir}_permissions"
diff "${got_dir}_permissions" "${want_dir}_permissions"
