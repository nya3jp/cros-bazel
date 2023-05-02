#!/bin/bash

# Copyright 2023 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

if [ "$#" -eq 0 ]; then
  echo "Got no args. Expected args to exist" >&2
  exit 1
fi

for arg in "$@"; do
  path=$(rlocation "${arg}")
  if [ ! -f "${path}" ]; then
    echo "${path} doesn't exist" >&2
    exit 1
  fi
done