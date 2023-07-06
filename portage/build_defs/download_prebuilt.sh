#!/bin/bash
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.


set -eu -o pipefail

GSUTIL="$1"
SRC="$2"
DST="$3"

protocol="${SRC%%://*}"

if [[ "${protocol}" = http ]] || [[ "${protocol}" = https ]]; then
  wget "${SRC}" -O "${DST}"
elif [[ "${protocol}" = "gs" ]]; then
  "${GSUTIL}" cp "${SRC}" "${DST}"
else
  cp "${SRC}" "${DST}"
fi
