#!/bin/sh
# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

exec bazel run -- @io_bazel_rules_go//go/tools/gopackagesdriver "$@"
