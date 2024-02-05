#!/bin/bash -ex
# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

generate_reclient_inputs

time tar \
  --format gnu \
  --sort name \
  --mtime "1970-1-1 00:00Z" \
  --numeric-owner \
  --create \
  /usr/bin/remote_toolchain_inputs \
  | \
  zstd -3 --long -T0 --force -o \
    "/mnt/host/.generate_reclient_inputs/output.tar.zst"
