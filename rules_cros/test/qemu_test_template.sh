#!/bin/bash
#
# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

BINARY=${1:-%PATH%}
echo "Testing that $BINARY outputs "Hello, world!""
if [ "$(%EMULATOR% "$BINARY")" != "Hello, world!" ]
then
  echo "$BINARY failed test"
  echo
  exit 1
fi
echo
