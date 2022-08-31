# Copyright 2022 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")

def prebuilts_dependencies():
    http_file(
        name = "arm64_generic_linux_headers_4_14_r52",
        downloaded_file_path = "linux-headers-4.14-r52.tbz2",
        sha256 = "e9d881c74ddfd6243866460506a9859919a3c1be7e1dc9b5777492f6a9ca03cf",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/board/arm64-generic/postsubmit-R106-14995.0.0-37502-8807992176298868113/packages/sys-kernel/linux-headers-4.14-r52.tbz2"],
    )
    http_file(
        name = "arm64_generic_gcc_libs_10_2_0_r4",
        downloaded_file_path = "gcc-libs-10.2.0-r4.tbz2",
        sha256 = "21dd868049fbd44bb4c6f4657b1f23f1a6e3879d72678bf6178ed580f4f8c8a8",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/board/arm64-generic/postsubmit-R106-14995.0.0-37502-8807992176298868113/packages/sys-libs/gcc-libs-10.2.0-r4.tbz2"],
    )
    http_file(
        name = "arm64_generic_libcxx_15_0_pre458507_r5",
        downloaded_file_path = "libcxx-15.0_pre458507-r5.tbz2",
        sha256 = "f1f0c27fbedd9f6476d796350fd47c07f54f79660f395cbaa14ea2ffdfc0412e",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/board/arm64-generic/postsubmit-R106-14995.0.0-37502-8807992176298868113/packages/sys-libs/libcxx-15.0_pre458507-r5.tbz2"],
    )
    http_file(
        name = "arm64_generic_llvm_libunwind_15_0_pre458507_r4",
        downloaded_file_path = "llvm-libunwind-15.0_pre458507-r4.tbz2",
        sha256 = "9f21b16420f77ae5e1e377ba17b4cdb6457e4395c52c8e06fa421c6873f82a94",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/board/arm64-generic/postsubmit-R106-14995.0.0-37502-8807992176298868113/packages/sys-libs/llvm-libunwind-15.0_pre458507-r4.tbz2"],
    )
    http_file(
        name = "amd64_host_binutils_2_36_1_r8",
        downloaded_file_path = "binutils-2.36.1-r8.tbz2",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/binutils-2.36.1-r8.tbz2"],
    )
    http_file(
        name = "amd64_host_compiler_rt_15_0_pre458507_r6",
        downloaded_file_path = "compiler-rt-15.0_pre458507-r6.tbz2",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/compiler-rt-15.0_pre458507-r6.tbz2"],
    )
    http_file(
        name = "amd64_host_gcc_10_2_0_r28",
        downloaded_file_path = "gcc-10.2.0-r28.tbz2",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/gcc-10.2.0-r28.tbz2"],
    )
    http_file(
        name = "amd64_host_gdb_9_2_20200923_r9",
        downloaded_file_path = "gdb-9.2.20200923-r9.tbz2",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/gdb-9.2.20200923-r9.tbz2"],
    )
    http_file(
        name = "amd64_host_glibc_2_33_r17",
        downloaded_file_path = "glibc-2.33-r17.tbz2",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/glibc-2.33-r17.tbz2"],
    )
    http_file(
        name = "amd64_host_go_1_18_r1",
        downloaded_file_path = "go-1.18-r1.tbz2",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/go-1.18-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_libcxx_15_0_pre458507_r5",
        downloaded_file_path = "libcxx-15.0_pre458507-r5.tbz2",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/libcxx-15.0_pre458507-r5.tbz2"],
    )
    http_file(
        name = "amd64_host_libxcrypt_4_4_28_r1",
        downloaded_file_path = "libxcrypt-4.4.28-r1.tbz2",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/libxcrypt-4.4.28-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_linux_headers_4_14_r52",
        downloaded_file_path = "linux-headers-4.14-r52.tbz2",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/linux-headers-4.14-r52.tbz2"],
    )
    http_file(
        name = "amd64_host_llvm_libunwind_15_0_pre458507_r4",
        downloaded_file_path = "llvm-libunwind-15.0_pre458507-r4.tbz2",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.07.20.214841/packages/cross-aarch64-cros-linux-gnu/llvm-libunwind-15.0_pre458507-r4.tbz2"],
    )
