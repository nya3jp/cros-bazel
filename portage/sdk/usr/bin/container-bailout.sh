#!/bin/bash
# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#
# Aborts the current container after printing an error message.

# Connect stdout/stderr to those of the init process.
exec >> /proc/1/fd/1 2>> /proc/1/fd/2

echo
echo "****************************************************************"
echo "container-bailout called!"
echo "****************************************************************"
echo
echo "reason: ${*:-unspecified}"
echo
echo "processes:"
ps auxwwf
echo
echo "aborting the container."

# Send SIGTERM to the container's init process.
if ! kill -s SIGTERM 1; then
  echo "container-bailout: Failed to kill init. Are we outside a container?"
  exit 1
fi

# Wait indefinitely until the container is turned down.
while :; do :; done
