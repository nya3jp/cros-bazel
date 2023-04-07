#!/bin/bash
#
# Copyright 2023 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

set -eu -o pipefail

if [[ "$#" -ne 1 ]]; then
	echo "Usage: <SDK version>" >&2
	exit 1
fi

VERSION="$1"

if [[ ! -v WORK_DIR ]]; then
	WORK_DIR="$(mktemp -d)"
fi

PACKAGES=(
	cross-aarch64-cros-linux-gnu/binutils
	cross-aarch64-cros-linux-gnu/compiler-rt
	cross-aarch64-cros-linux-gnu/gcc
	cross-aarch64-cros-linux-gnu/gdb
	cross-aarch64-cros-linux-gnu/glibc
	cross-aarch64-cros-linux-gnu/go
	cross-aarch64-cros-linux-gnu/libcxx
	cross-aarch64-cros-linux-gnu/libxcrypt
	cross-aarch64-cros-linux-gnu/linux-headers
	cross-aarch64-cros-linux-gnu/llvm-libunwind

	cross-armv7a-cros-linux-gnueabihf/binutils
	cross-armv7a-cros-linux-gnueabihf/compiler-rt
	cross-armv7a-cros-linux-gnueabihf/gcc
	cross-armv7a-cros-linux-gnueabihf/gdb
	cross-armv7a-cros-linux-gnueabihf/glibc
	cross-armv7a-cros-linux-gnueabihf/go
	cross-armv7a-cros-linux-gnueabihf/libcxx
	cross-armv7a-cros-linux-gnueabihf/libxcrypt
	cross-armv7a-cros-linux-gnueabihf/linux-headers
	cross-armv7a-cros-linux-gnueabihf/llvm-libunwind

	cross-x86_64-cros-linux-gnu/binutils
	cross-x86_64-cros-linux-gnu/gcc
	cross-x86_64-cros-linux-gnu/gdb
	cross-x86_64-cros-linux-gnu/glibc
	cross-x86_64-cros-linux-gnu/go
	cross-x86_64-cros-linux-gnu/libcxx
	cross-x86_64-cros-linux-gnu/libxcrypt
	cross-x86_64-cros-linux-gnu/linux-headers
	cross-x86_64-cros-linux-gnu/llvm-libunwind
	dev-lang/rust
	dev-embedded/coreboot-sdk
	dev-embedded/hps-sdk
)

for PACKAGE in "${PACKAGES[@]}"; do
	PACKAGE_DIR="${WORK_DIR}/${PACKAGE}"
	mkdir -p "${PACKAGE_DIR}"

	gsutil cp -n \
		"gs://chromeos-prebuilt/host/amd64/amd64-host/chroot-${VERSION}/packages/${PACKAGE}-*" \
		"${PACKAGE_DIR}/"
done


while read -r FULL_PATH
do
	PACKAGE="$(dirname "${FULL_PATH}")"
	FILE_NAME="$(basename "${FULL_PATH}")"

	WITHOUT_EXT="${FILE_NAME%.*}"
	CATEGORY="${PACKAGE%\/**}"

	echo "http_file(
	name = \"amd64_host_${VERSION//./_}_${CATEGORY//[-\/]/_}_${WITHOUT_EXT//[.-]/_}\",
	downloaded_file_path = \"${FILE_NAME}\",
	sha256 = \"$(sha256sum "${WORK_DIR}/${FULL_PATH}" | cut -d' ' -f 1)\",
	urls = [\"https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-${VERSION}/packages/${CATEGORY}/${FILE_NAME}\"],
)"
done < <(find "${WORK_DIR}/" -mindepth 1 -type f -name "*.tbz2" -printf "%P\n" | sort)
