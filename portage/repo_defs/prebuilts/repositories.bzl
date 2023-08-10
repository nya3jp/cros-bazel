# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
load("@bazel_tools//tools/build_defs/repo:http.bzl", _http_file = "http_file")

def prebuilts_dependencies(http_file = _http_file):
    # ~/cros-bazel/src/bazel/tools/sdk_repos.sh 2023.04.05.144808
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_aarch64_cros_linux_gnu_binutils_2_36_1_r10",
        downloaded_file_path = "binutils-2.36.1-r10.tbz2",
        sha256 = "85f8a32a9096761a247e1dc4daf29e02599b12fd84673be9fabab9158f88a385",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-aarch64-cros-linux-gnu/binutils-2.36.1-r10.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_aarch64_cros_linux_gnu_compiler_rt_16_0_pre475826_r6",
        downloaded_file_path = "compiler-rt-16.0_pre475826-r6.tbz2",
        sha256 = "f8c120e47dba78397395a1f328dfd65a35a6cf7fd93de1c79c2ad91bf4ea59c9",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-aarch64-cros-linux-gnu/compiler-rt-16.0_pre475826-r6.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_aarch64_cros_linux_gnu_gcc_10_2_0_r30",
        downloaded_file_path = "gcc-10.2.0-r30.tbz2",
        sha256 = "6e05fd2c5e247475611e27aa517c0dbac1f36890d1bee98650d2b20fd5e7e090",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-aarch64-cros-linux-gnu/gcc-10.2.0-r30.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_aarch64_cros_linux_gnu_gdb_11_2_r3",
        downloaded_file_path = "gdb-11.2-r3.tbz2",
        sha256 = "3717db078fab85244fe4133804f781796397414d6566ae6323c93ec6f7659be8",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-aarch64-cros-linux-gnu/gdb-11.2-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_aarch64_cros_linux_gnu_glibc_2_35_r17",
        downloaded_file_path = "glibc-2.35-r17.tbz2",
        sha256 = "9701dd4d12221cd2cc6eb8f44cc6da49a34b0fb6f14361a2909b7200cdcc1cca",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-aarch64-cros-linux-gnu/glibc-2.35-r17.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_aarch64_cros_linux_gnu_go_1_20_2_r2",
        downloaded_file_path = "go-1.20.2-r2.tbz2",
        sha256 = "b920acc4dcd8c664ce46240c392f09b2ff49789a08f062d7401d51f12c9fc372",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-aarch64-cros-linux-gnu/go-1.20.2-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_aarch64_cros_linux_gnu_libcxx_16_0_pre475826_r5",
        downloaded_file_path = "libcxx-16.0_pre475826-r5.tbz2",
        sha256 = "ef87fad817c516b2788b240bd85f0719622cbcd810bc65f6d3686e87e09ac385",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-aarch64-cros-linux-gnu/libcxx-16.0_pre475826-r5.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_aarch64_cros_linux_gnu_libxcrypt_4_4_28_r2",
        downloaded_file_path = "libxcrypt-4.4.28-r2.tbz2",
        sha256 = "90fe3f66f2d5b9e662dafaab45748013048053303afa7556388733e9c119a100",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-aarch64-cros-linux-gnu/libxcrypt-4.4.28-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_aarch64_cros_linux_gnu_linux_headers_4_14_r61",
        downloaded_file_path = "linux-headers-4.14-r61.tbz2",
        sha256 = "8344e3a2a59bd26a8a9348fbb4b8c013497bb20e40f996a652527d0df8b14412",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-aarch64-cros-linux-gnu/linux-headers-4.14-r61.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_aarch64_cros_linux_gnu_llvm_libunwind_16_0_pre475826_r5",
        downloaded_file_path = "llvm-libunwind-16.0_pre475826-r5.tbz2",
        sha256 = "f342f9b8178ec58eed3ca26e2a8be78e5fe402b53214b0b7e988d5e98339b04d",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-aarch64-cros-linux-gnu/llvm-libunwind-16.0_pre475826-r5.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_armv7a_cros_linux_gnueabihf_binutils_2_36_1_r10",
        downloaded_file_path = "binutils-2.36.1-r10.tbz2",
        sha256 = "2764aceecd01a5ef4f8a62d18bf023f7008554fda723e9cfed0bc339303b9e0c",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-armv7a-cros-linux-gnueabihf/binutils-2.36.1-r10.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_armv7a_cros_linux_gnueabihf_compiler_rt_16_0_pre475826_r6",
        downloaded_file_path = "compiler-rt-16.0_pre475826-r6.tbz2",
        sha256 = "24d37b83282163428a356f088a1cf134dc6f6984d185f9beb4fc58d5cb995cf6",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-armv7a-cros-linux-gnueabihf/compiler-rt-16.0_pre475826-r6.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_armv7a_cros_linux_gnueabihf_gcc_10_2_0_r30",
        downloaded_file_path = "gcc-10.2.0-r30.tbz2",
        sha256 = "b617206363d65d00ebffb930314c8a89c0dcd77cab9f02b6c825f88ce067ad6e",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-armv7a-cros-linux-gnueabihf/gcc-10.2.0-r30.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_armv7a_cros_linux_gnueabihf_gdb_11_2_r3",
        downloaded_file_path = "gdb-11.2-r3.tbz2",
        sha256 = "18db01e843111c61c1c4261e31db1b2ab4e7614792bc239f53fe8d7a4ec6b55b",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-armv7a-cros-linux-gnueabihf/gdb-11.2-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_armv7a_cros_linux_gnueabihf_glibc_2_35_r17",
        downloaded_file_path = "glibc-2.35-r17.tbz2",
        sha256 = "fb02cd1b605083396c15ed7c8f88cb03cb2c61d3bf9d46b9bc121e33d76614cd",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-armv7a-cros-linux-gnueabihf/glibc-2.35-r17.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_armv7a_cros_linux_gnueabihf_go_1_20_2_r2",
        downloaded_file_path = "go-1.20.2-r2.tbz2",
        sha256 = "1613ae679d2757fc75323ed542e5e946cdbb397a95a0c1564410f9a89c4b1a0a",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-armv7a-cros-linux-gnueabihf/go-1.20.2-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_armv7a_cros_linux_gnueabihf_libcxx_16_0_pre475826_r5",
        downloaded_file_path = "libcxx-16.0_pre475826-r5.tbz2",
        sha256 = "1a0c1c66863e1309a74808b9c102c0987d10c0ac563f20cf9a0cf7624f5ae9c2",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-armv7a-cros-linux-gnueabihf/libcxx-16.0_pre475826-r5.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_armv7a_cros_linux_gnueabihf_libxcrypt_4_4_28_r2",
        downloaded_file_path = "libxcrypt-4.4.28-r2.tbz2",
        sha256 = "844f60657beeffdacedd376110b56f3e2146779d1a46b9e42d020d2e3968ad03",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-armv7a-cros-linux-gnueabihf/libxcrypt-4.4.28-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_armv7a_cros_linux_gnueabihf_linux_headers_4_14_r61",
        downloaded_file_path = "linux-headers-4.14-r61.tbz2",
        sha256 = "9061909f9c09a32c7dc2fe64d1623d5125b32cbb76c414edd1849cb6a6d07fe3",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-armv7a-cros-linux-gnueabihf/linux-headers-4.14-r61.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_armv7a_cros_linux_gnueabihf_llvm_libunwind_16_0_pre475826_r5",
        downloaded_file_path = "llvm-libunwind-16.0_pre475826-r5.tbz2",
        sha256 = "4528edd2f4688065693a661d8473c0f8bbb2e45f0bcc5f7d2d80402d72500528",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-armv7a-cros-linux-gnueabihf/llvm-libunwind-16.0_pre475826-r5.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_x86_64_cros_linux_gnu_binutils_2_36_1_r10",
        downloaded_file_path = "binutils-2.36.1-r10.tbz2",
        sha256 = "c58124fb015702ab66d80f395c080d005c0a732bdc65107a9d64021994c86511",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-x86_64-cros-linux-gnu/binutils-2.36.1-r10.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_x86_64_cros_linux_gnu_gcc_10_2_0_r30",
        downloaded_file_path = "gcc-10.2.0-r30.tbz2",
        sha256 = "379c38b4dd1de0c32c73446e26317f7a6a0587205931400f428ecc971f440e07",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-x86_64-cros-linux-gnu/gcc-10.2.0-r30.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_x86_64_cros_linux_gnu_gdb_11_2_r3",
        downloaded_file_path = "gdb-11.2-r3.tbz2",
        sha256 = "f47fec049205e537e9318e897a16a1357e957e37c274b2bbe0a501a1518bcace",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-x86_64-cros-linux-gnu/gdb-11.2-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_x86_64_cros_linux_gnu_glibc_2_35_r17",
        downloaded_file_path = "glibc-2.35-r17.tbz2",
        sha256 = "c62a1df4c14f18a09057c0d1fdda1d5205bae0fb9cff45048980c3756b5b67e4",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-x86_64-cros-linux-gnu/glibc-2.35-r17.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_x86_64_cros_linux_gnu_go_1_20_2_r2",
        downloaded_file_path = "go-1.20.2-r2.tbz2",
        sha256 = "fa0f7b98a3b3b7685d9f47ab57f82d300afeb2c8b3fc4f69729cb037b1efdf7c",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-x86_64-cros-linux-gnu/go-1.20.2-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_x86_64_cros_linux_gnu_libcxx_16_0_pre475826_r5",
        downloaded_file_path = "libcxx-16.0_pre475826-r5.tbz2",
        sha256 = "e83539a849e0139bed2071405708d0a3f9bcb1cc99daf14e396a35fe623165c5",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-x86_64-cros-linux-gnu/libcxx-16.0_pre475826-r5.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_x86_64_cros_linux_gnu_libxcrypt_4_4_28_r2",
        downloaded_file_path = "libxcrypt-4.4.28-r2.tbz2",
        sha256 = "fca35c6b14867ad87bd0191f0518e616da1882015abb9b1ac7983c30fd0a70ec",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-x86_64-cros-linux-gnu/libxcrypt-4.4.28-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_x86_64_cros_linux_gnu_linux_headers_4_14_r61",
        downloaded_file_path = "linux-headers-4.14-r61.tbz2",
        sha256 = "73d4f0dcfa213ebe49bead5c4283a80f91ff3894a3806709475d3470fb623487",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-x86_64-cros-linux-gnu/linux-headers-4.14-r61.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_cross_x86_64_cros_linux_gnu_llvm_libunwind_16_0_pre475826_r5",
        downloaded_file_path = "llvm-libunwind-16.0_pre475826-r5.tbz2",
        sha256 = "a46c4cd5142b293569d9c93bae32ab0da6594f9f9a9e1a0aac4957dc77a94164",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/cross-x86_64-cros-linux-gnu/llvm-libunwind-16.0_pre475826-r5.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_dev_embedded_coreboot_sdk_0_0_1_r119",
        downloaded_file_path = "coreboot-sdk-0.0.1-r119.tbz2",
        sha256 = "91ffda7914a48673023b021cda7845bf16f6b2a991befb8258f027750f1a8574",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/dev-embedded/coreboot-sdk-0.0.1-r119.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_dev_embedded_hps_sdk_0_0_1_r5",
        downloaded_file_path = "hps-sdk-0.0.1-r5.tbz2",
        sha256 = "8dcfc2309690b6eaa3d2c93df65d2ba5af033398e7c6d321585dadc6d4b7a541",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/dev-embedded/hps-sdk-0.0.1-r5.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_dev_lang_rust_1_68_0",
        downloaded_file_path = "rust-1.68.0.tbz2",
        sha256 = "70c63844c24410bf2ffee1b80d6877d499429f147c3b7a6ddfc8e88c5b0b563a",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/dev-lang/rust-1.68.0.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_dev_lang_rust_bootstrap_1_67_0",
        downloaded_file_path = "rust-bootstrap-1.67.0.tbz2",
        sha256 = "d814493c05286bbab5a8502ce2eecdfed8becd7b36d7e6dd397a7d9ce9a6da9e",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/dev-lang/rust-bootstrap-1.67.0.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_04_05_144808_dev_lang_rust_host_1_68_0",
        downloaded_file_path = "rust-host-1.68.0.tbz2",
        sha256 = "a2593b64bd0a4451661c44a3cd8409224857900fac2c77fd71d94e94ac411f62",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.04.05.144808/packages/dev-lang/rust-host-1.68.0.tbz2"],
    )

    # portage/tools/sdk_repos.sh 2023.08.08.170046
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_aarch64_cros_linux_gnu_binutils_2_39_r3",
        downloaded_file_path = "binutils-2.39-r3.tbz2",
        sha256 = "6c4756733bad0ca5468c96f7af88591f0909d8a305c2a13816df116b6917e6cd",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-aarch64-cros-linux-gnu/binutils-2.39-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_aarch64_cros_linux_gnu_compiler_rt_17_0_pre496208_r4",
        downloaded_file_path = "compiler-rt-17.0_pre496208-r4.tbz2",
        sha256 = "98c7ef6c6cc8d2e22eec8c80597755b901d90243515de4f603fa04a060be0db3",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-aarch64-cros-linux-gnu/compiler-rt-17.0_pre496208-r4.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_aarch64_cros_linux_gnu_gcc_10_2_0_r34",
        downloaded_file_path = "gcc-10.2.0-r34.tbz2",
        sha256 = "b92088602dd2c2149e10efef58ecd1a8445cf3125cf928417c59484333d37066",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-aarch64-cros-linux-gnu/gcc-10.2.0-r34.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_aarch64_cros_linux_gnu_gdb_9_2_20200923_r10",
        downloaded_file_path = "gdb-9.2.20200923-r10.tbz2",
        sha256 = "e0e5d50b249358ea969b0aca7f1dc6f89049d5cb5a40bee1b727b54d5e24018a",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-aarch64-cros-linux-gnu/gdb-9.2.20200923-r10.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_aarch64_cros_linux_gnu_glibc_2_35_r22",
        downloaded_file_path = "glibc-2.35-r22.tbz2",
        sha256 = "306733f71c97c79085201e810b2101eaa659718bea6fec094e217f3c0e1137b5",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-aarch64-cros-linux-gnu/glibc-2.35-r22.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_aarch64_cros_linux_gnu_go_1_20_5_r1",
        downloaded_file_path = "go-1.20.5-r1.tbz2",
        sha256 = "9b876207df5fbae7deb17ef72bf0994e47b79906489e88145fb396c8161d322d",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-aarch64-cros-linux-gnu/go-1.20.5-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_aarch64_cros_linux_gnu_libcxx_17_0_pre496208_r13",
        downloaded_file_path = "libcxx-17.0_pre496208-r13.tbz2",
        sha256 = "38ddc0f32f916060db7f3c4e1bad97a9e0a99d9ac1fb65062f7134e13f2f980a",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-aarch64-cros-linux-gnu/libcxx-17.0_pre496208-r13.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_aarch64_cros_linux_gnu_libxcrypt_4_4_28_r2",
        downloaded_file_path = "libxcrypt-4.4.28-r2.tbz2",
        sha256 = "e03b3280595190efb9ead88d6dee8b58a667e8249f4367a22975f7f2a6afc10f",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-aarch64-cros-linux-gnu/libxcrypt-4.4.28-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_aarch64_cros_linux_gnu_linux_headers_4_14_r2",
        downloaded_file_path = "linux-headers-4.14-r2.tbz2",
        sha256 = "e8aaa206e595dd09d5faf07dcbe8dafdbab06cc2870d870b5b254cc5dfdd0f45",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-aarch64-cros-linux-gnu/linux-headers-4.14-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_aarch64_cros_linux_gnu_llvm_libunwind_17_0_pre496208_r7",
        downloaded_file_path = "llvm-libunwind-17.0_pre496208-r7.tbz2",
        sha256 = "ad764dc1749dbbf31839aa935472cfe5c1245e3e76a3c02b9a824cb101e4ad26",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-aarch64-cros-linux-gnu/llvm-libunwind-17.0_pre496208-r7.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_armv7a_cros_linux_gnueabihf_binutils_2_39_r3",
        downloaded_file_path = "binutils-2.39-r3.tbz2",
        sha256 = "6e5a4d5d04e125872887664299bfa389817fb250e32b6cd325a9a024577ea2c4",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-armv7a-cros-linux-gnueabihf/binutils-2.39-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_armv7a_cros_linux_gnueabihf_compiler_rt_17_0_pre496208_r4",
        downloaded_file_path = "compiler-rt-17.0_pre496208-r4.tbz2",
        sha256 = "6885a3db38c4ef05e729748ea23015ea45194ace8d5ec397c999abe8775541ad",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-armv7a-cros-linux-gnueabihf/compiler-rt-17.0_pre496208-r4.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_armv7a_cros_linux_gnueabihf_gcc_10_2_0_r34",
        downloaded_file_path = "gcc-10.2.0-r34.tbz2",
        sha256 = "6cc13d0afa7ac9c5c71fd00c3de646c9ea87de0e51e101f5ab3fbbf8d7b5c251",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-armv7a-cros-linux-gnueabihf/gcc-10.2.0-r34.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_armv7a_cros_linux_gnueabihf_gdb_9_2_20200923_r10",
        downloaded_file_path = "gdb-9.2.20200923-r10.tbz2",
        sha256 = "d21fb3e1a09f566943c2dab1ecb072b4cc79bf1581e2c999746a2b22afe23160",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-armv7a-cros-linux-gnueabihf/gdb-9.2.20200923-r10.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_armv7a_cros_linux_gnueabihf_glibc_2_35_r22",
        downloaded_file_path = "glibc-2.35-r22.tbz2",
        sha256 = "5461660e6e5eeec1d8ee8725d39765fd89c9ba54f05f7398eb4725075fb06eb8",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-armv7a-cros-linux-gnueabihf/glibc-2.35-r22.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_armv7a_cros_linux_gnueabihf_go_1_20_5_r1",
        downloaded_file_path = "go-1.20.5-r1.tbz2",
        sha256 = "9583e3113960070989cc80ad58700019439e8cfd5ee6a4393f9fb950fa5f6194",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-armv7a-cros-linux-gnueabihf/go-1.20.5-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_armv7a_cros_linux_gnueabihf_libcxx_17_0_pre496208_r13",
        downloaded_file_path = "libcxx-17.0_pre496208-r13.tbz2",
        sha256 = "baac71bbadb87593da9a03082cf7ec198ad37916d41a94626dc4ac8161d9cc0a",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-armv7a-cros-linux-gnueabihf/libcxx-17.0_pre496208-r13.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_armv7a_cros_linux_gnueabihf_libxcrypt_4_4_28_r2",
        downloaded_file_path = "libxcrypt-4.4.28-r2.tbz2",
        sha256 = "0cecd76dd5a305cf8fd2057114e99837e70663a34bf85ea7564768c40fb0be68",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-armv7a-cros-linux-gnueabihf/libxcrypt-4.4.28-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_armv7a_cros_linux_gnueabihf_linux_headers_4_14_r2",
        downloaded_file_path = "linux-headers-4.14-r2.tbz2",
        sha256 = "c87cdd7f645436168b89f3d7fe902678a82b8cc212c9922299e5f52560139c90",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-armv7a-cros-linux-gnueabihf/linux-headers-4.14-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_armv7a_cros_linux_gnueabihf_llvm_libunwind_17_0_pre496208_r7",
        downloaded_file_path = "llvm-libunwind-17.0_pre496208-r7.tbz2",
        sha256 = "d43b970a09c694adf9be8f7da9fc37aac725426496f5b52a6bb73288b244c855",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-armv7a-cros-linux-gnueabihf/llvm-libunwind-17.0_pre496208-r7.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_x86_64_cros_linux_gnu_binutils_2_39_r3",
        downloaded_file_path = "binutils-2.39-r3.tbz2",
        sha256 = "07f2346295fd271a60ca03db3f618296004f9cdffe2392f9345109759b9b6526",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-x86_64-cros-linux-gnu/binutils-2.39-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_x86_64_cros_linux_gnu_gcc_10_2_0_r34",
        downloaded_file_path = "gcc-10.2.0-r34.tbz2",
        sha256 = "5f7b01a9c4474a72c3b54ee19af5440cff4680cc190e61c11a63d218aa04f781",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-x86_64-cros-linux-gnu/gcc-10.2.0-r34.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_x86_64_cros_linux_gnu_gdb_9_2_20200923_r10",
        downloaded_file_path = "gdb-9.2.20200923-r10.tbz2",
        sha256 = "b423608796656c5d6b53fa4e09af09cb6a29e825da83d16585a3dfd7fbcdc442",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-x86_64-cros-linux-gnu/gdb-9.2.20200923-r10.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_x86_64_cros_linux_gnu_glibc_2_35_r22",
        downloaded_file_path = "glibc-2.35-r22.tbz2",
        sha256 = "4c3d116fa377adfbd3f3db52be9256082d3beda1c55f47fdf323f5815cd8e9c8",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-x86_64-cros-linux-gnu/glibc-2.35-r22.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_x86_64_cros_linux_gnu_go_1_20_5_r1",
        downloaded_file_path = "go-1.20.5-r1.tbz2",
        sha256 = "8056be2e8b5ec7c158c6ccdd962dd5e7704477a9a6e9ffb2e737e3fb7de21be3",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-x86_64-cros-linux-gnu/go-1.20.5-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_x86_64_cros_linux_gnu_libcxx_17_0_pre496208_r13",
        downloaded_file_path = "libcxx-17.0_pre496208-r13.tbz2",
        sha256 = "138c29b4d1e366e6b1f2dd06f1e05424c03aa085625018188a6ff5fcfe8f5068",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-x86_64-cros-linux-gnu/libcxx-17.0_pre496208-r13.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_x86_64_cros_linux_gnu_libxcrypt_4_4_28_r2",
        downloaded_file_path = "libxcrypt-4.4.28-r2.tbz2",
        sha256 = "9474ce0eca2340cceb41174e5548d85ee00daba2c4ecb4e1e33115abcc47fde9",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-x86_64-cros-linux-gnu/libxcrypt-4.4.28-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_x86_64_cros_linux_gnu_linux_headers_4_14_r2",
        downloaded_file_path = "linux-headers-4.14-r2.tbz2",
        sha256 = "4851cf63d88b46c1bdf2fe55f3f47553091493a9b842b3a22eff1d072382d68e",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-x86_64-cros-linux-gnu/linux-headers-4.14-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_cross_x86_64_cros_linux_gnu_llvm_libunwind_17_0_pre496208_r7",
        downloaded_file_path = "llvm-libunwind-17.0_pre496208-r7.tbz2",
        sha256 = "ae4a150a1431f20f378e0c4955341ab8edb1786f0be7a9bf61bccba09d1f2a83",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/cross-x86_64-cros-linux-gnu/llvm-libunwind-17.0_pre496208-r7.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_dev_embedded_coreboot_sdk_0_0_1_r120",
        downloaded_file_path = "coreboot-sdk-0.0.1-r120.tbz2",
        sha256 = "6b161b2d6e8d897fd79225c764bc3db8bf5514442d2193e0a702b511ad7e3403",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/dev-embedded/coreboot-sdk-0.0.1-r120.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_dev_embedded_hps_sdk_0_0_1_r7",
        downloaded_file_path = "hps-sdk-0.0.1-r7.tbz2",
        sha256 = "189205a75ef0e8b0c6a5f27846f2ae53215a7b0a2f58909b9472a13aff66bfea",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/dev-embedded/hps-sdk-0.0.1-r7.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_dev_lang_rust_1_70_0",
        downloaded_file_path = "rust-1.70.0.tbz2",
        sha256 = "d302c07e2078edcec599577878154829228600e93cea7c833fd3e76b33b007e2",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/dev-lang/rust-1.70.0.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_dev_lang_rust_bootstrap_1_69_0",
        downloaded_file_path = "rust-bootstrap-1.69.0.tbz2",
        sha256 = "523b322312f5f1cea0ebe2543234741997e03e104c38b21be317ab3a22a51941",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/dev-lang/rust-bootstrap-1.69.0.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_dev_lang_rust_host_1_70_0",
        downloaded_file_path = "rust-host-1.70.0.tbz2",
        sha256 = "dbbd7aa48e93b33ac613f59edb6f717dcfa9b0a2e35d2fc48dfb3cfb1f22967d",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/dev-lang/rust-host-1.70.0.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_08_170046_dev_util_glib_utils_2_74_1",
        downloaded_file_path = "glib-utils-2.74.1.tbz2",
        sha256 = "e8674c7f4bc0867467610c105ea043d97a548a8885505feb6a48df54e804b1fd",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.08.170046/packages/dev-util/glib-utils-2.74.1.tbz2"],
    )

    # Use GN binary built without rpmalloc to avoid crash bug.
    # TODO(b/273830995): Remove this.
    http_file(
        name = "gn_without_rpmalloc",
        downloaded_file_path = "gn_without_rpmalloc",
        sha256 = "46ae28050ac648738a908284807184eb3dddd176b7a8054db63c14e8757d5b81",
        urls = ["https://commondatastorage.googleapis.com/chromeos-throw-away-bucket/cros-bazel/gn_without_rpmalloc"],
    )

    # Force using the new version automake.
    # TODO(b/295260057): Remove this after updating SDK archive.
    http_file(
        name = "automake-1.16.5-r1",
        downloaded_file_path = "automake-1.16.5-r1.tbz2",
        sha256 = "0303b2f4e3f684660e7d0156ff0cbe04b5b8be0e2326cfc9167d6e5912d01e8f",
        urls = ["https://commondatastorage.googleapis.com/chromeos-throw-away-bucket/cros-bazel/automake-1.16.5-r1.tbz2"],
    )
