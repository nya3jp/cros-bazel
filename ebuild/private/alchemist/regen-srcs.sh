#!/bin/bash
#
# Copyright 2023 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

set -eu -o pipefail

cd "$(dirname -- "$0")"

(
	echo "# AUTO GENERATED DO NOT EDIT!"
	echo "# Regenerate this file using ./$(basename -- "$0")"
	echo "# It should be regenerated each time a file is added or removed."

	echo "ALCHEMIST_SRCS = ["
	(
		# Add the workspace root Cargo.toml.
		# TODO: Check if `cargo_bootstrap_repository` derives this from the project's
		# Cargo.toml.
		echo '    Label("//:Cargo.toml"),'
		find src -type f \
			-printf '    Label("//bazel/ebuild/private/alchemist:%p"),\n'

		find ../common/standard/version \
			-type f \
			! \( -name '*.go' -or -name 'BUILD.bazel' \) \
			-printf '    Label("//bazel/ebuild/private/common/standard/version:%P"),\n'
	) | LC_ALL=C sort

	echo "]"
) > ./src.bzl

echo Done
