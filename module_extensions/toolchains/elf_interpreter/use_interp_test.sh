#!/bin/bash -e

# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.


BIN=cros/bazel/module_extensions/toolchains/elf_interpreter/use_interp_test_cc
PATH=$(rlocation "${BIN}")

WANT="Hello, World!"

GOT=$("${PATH}" foo) || (
  RETCODE="$?"
  echo "${PATH} exited with status ${RETCODE}" >&2
  exit "${RETCODE}"
)

if [[ "${GOT}" != "${WANT}"* ]]; then
  echo "Got '${GOT}', want it to start with '${WANT}'" >&2
  exit 1
fi
