#!/bin/bash
# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# Usage:
#   ./test_script.sh <EXIT-CODE|SIGNAL-NAME> <OUT-MESSAGE> <ERR-MESSAGE>

echo stdout "$2"
echo stderr "$3" 1>&2

# Treat a nuumeric argument as an exit code and string as a signal name.
if [[ "$1" =~ ^[0-9]*$ ]]; then
    exit "$1"
else
    kill "-${1}" $$
fi
