#!/bin/bash -eu

# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

SITE_CUSTOMIZE_RPATH='cros/bazel/python/toolchains/sitecustomize.py'
SITE_CUSTOMIZE_DIR="$(dirname "$(rlocation "${SITE_CUSTOMIZE_RPATH}")")"
INTERP="$(rlocation python_interpreter/bin/python3)"
PYTHONPATH="${SITE_CUSTOMIZE_DIR}:${PYTHONPATH:-}" exec "${INTERP}" "$@"
