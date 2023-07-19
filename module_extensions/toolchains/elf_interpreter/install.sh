#!/bin/bash -e

# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

SRC=cros/bazel/module_extensions/toolchains/elf_interpreter/elf_interpreter
DST="${1:-/tmp/elf_interpreter}"

sudo install --mode 0755 --owner=root "$(rlocation "${SRC}")" "${DST}"
