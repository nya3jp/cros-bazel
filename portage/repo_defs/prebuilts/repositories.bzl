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

    # portage/tools/sdk_repos.sh 2023.08.03.170038
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_aarch64_cros_linux_gnu_binutils_2_39_r3",
        downloaded_file_path = "binutils-2.39-r3.tbz2",
        sha256 = "2faf3bfe5d0726468ec7b6e4c6f2f9732786beccc9e678b32a6088ef0895cee3",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-aarch64-cros-linux-gnu/binutils-2.39-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_aarch64_cros_linux_gnu_compiler_rt_17_0_pre496208_r4",
        downloaded_file_path = "compiler-rt-17.0_pre496208-r4.tbz2",
        sha256 = "af1411a9e523e36d1acb913d9af95801ef08f345102252fc81d44464e88880c4",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-aarch64-cros-linux-gnu/compiler-rt-17.0_pre496208-r4.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_aarch64_cros_linux_gnu_gcc_10_2_0_r34",
        downloaded_file_path = "gcc-10.2.0-r34.tbz2",
        sha256 = "18bec8ac1724dd63b2d847c3b1701edf935917f59bec60d691d93ef4e4238ca8",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-aarch64-cros-linux-gnu/gcc-10.2.0-r34.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_aarch64_cros_linux_gnu_gdb_9_2_20200923_r10",
        downloaded_file_path = "gdb-9.2.20200923-r10.tbz2",
        sha256 = "d38e7d1d9678e5adb1b698ee0fa470e1644b733eaf014f9422d7ab8aa8ca3023",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-aarch64-cros-linux-gnu/gdb-9.2.20200923-r10.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_aarch64_cros_linux_gnu_glibc_2_35_r22",
        downloaded_file_path = "glibc-2.35-r22.tbz2",
        sha256 = "0f7eda89e1b0cac9a294faa61933d163689ca7b5271c2a5c4aa872e24aa39781",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-aarch64-cros-linux-gnu/glibc-2.35-r22.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_aarch64_cros_linux_gnu_go_1_20_5_r1",
        downloaded_file_path = "go-1.20.5-r1.tbz2",
        sha256 = "6d046d0c32e430ffab4e37d326d4ae5712273d1665bbc83b9c9d3ef9c5a7a840",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-aarch64-cros-linux-gnu/go-1.20.5-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_aarch64_cros_linux_gnu_libcxx_9999",
        downloaded_file_path = "libcxx-9999.tbz2",
        sha256 = "e82ee62d0b49e7c49f11555849a841a8355408f5f0e32f0b9ddac2e98b5170e0",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-aarch64-cros-linux-gnu/libcxx-9999.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_aarch64_cros_linux_gnu_libxcrypt_4_4_28_r2",
        downloaded_file_path = "libxcrypt-4.4.28-r2.tbz2",
        sha256 = "2970a59bf9ee8bfe0333c5ec49b2a591891490cce13f053d53ae9de68509a2d1",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-aarch64-cros-linux-gnu/libxcrypt-4.4.28-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_aarch64_cros_linux_gnu_linux_headers_4_14_r2",
        downloaded_file_path = "linux-headers-4.14-r2.tbz2",
        sha256 = "f72d86979b170da09b629a7fa5aba84f9c5b1afbbfbed47538a2f2f0ba2a96c7",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-aarch64-cros-linux-gnu/linux-headers-4.14-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_aarch64_cros_linux_gnu_llvm_libunwind_17_0_pre496208_r7",
        downloaded_file_path = "llvm-libunwind-17.0_pre496208-r7.tbz2",
        sha256 = "6fd57bed7a6d81ffd6e465dec6709e230bb68b5e0490323ad3a13be3d313b5fd",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-aarch64-cros-linux-gnu/llvm-libunwind-17.0_pre496208-r7.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_armv7a_cros_linux_gnueabihf_binutils_2_39_r3",
        downloaded_file_path = "binutils-2.39-r3.tbz2",
        sha256 = "eb3f7b30f3988052d17a57b92bc5f2ed51885282b0cfed9c58c413c1eae14617",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-armv7a-cros-linux-gnueabihf/binutils-2.39-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_armv7a_cros_linux_gnueabihf_compiler_rt_17_0_pre496208_r4",
        downloaded_file_path = "compiler-rt-17.0_pre496208-r4.tbz2",
        sha256 = "5648845c71fc113316c16ce8ee3b102cdadda50f59539cb0f069ab9fdd2262c9",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-armv7a-cros-linux-gnueabihf/compiler-rt-17.0_pre496208-r4.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_armv7a_cros_linux_gnueabihf_gcc_10_2_0_r34",
        downloaded_file_path = "gcc-10.2.0-r34.tbz2",
        sha256 = "efb8e6bd17fa4f99d437043dbc21e174438ff656d3f668fcf4f2c4f21e0dcc06",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-armv7a-cros-linux-gnueabihf/gcc-10.2.0-r34.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_armv7a_cros_linux_gnueabihf_gdb_9_2_20200923_r10",
        downloaded_file_path = "gdb-9.2.20200923-r10.tbz2",
        sha256 = "ae4c6ed31502209cda04170f3f3c67a88295eeb279d7929a242bc831e5a97b13",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-armv7a-cros-linux-gnueabihf/gdb-9.2.20200923-r10.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_armv7a_cros_linux_gnueabihf_glibc_2_35_r22",
        downloaded_file_path = "glibc-2.35-r22.tbz2",
        sha256 = "7c7b67b2ae4c3cfde632b4d1e3fb3730e159873381172230749a724d5c9b00d3",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-armv7a-cros-linux-gnueabihf/glibc-2.35-r22.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_armv7a_cros_linux_gnueabihf_go_1_20_5_r1",
        downloaded_file_path = "go-1.20.5-r1.tbz2",
        sha256 = "c2bc384f5445179ddce61cb0521ade23a4f2ec2c03295f191f24445c980ee96c",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-armv7a-cros-linux-gnueabihf/go-1.20.5-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_armv7a_cros_linux_gnueabihf_libcxx_9999",
        downloaded_file_path = "libcxx-9999.tbz2",
        sha256 = "54627ac1f0aa8d05cd8459d1b4fdfc37f5ad0701390c05845cbe983411911eab",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-armv7a-cros-linux-gnueabihf/libcxx-9999.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_armv7a_cros_linux_gnueabihf_libxcrypt_4_4_28_r2",
        downloaded_file_path = "libxcrypt-4.4.28-r2.tbz2",
        sha256 = "3a9b4da3b3d06764f0831a82f73efe910676966c1063c59bfb0d9b81109be3ac",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-armv7a-cros-linux-gnueabihf/libxcrypt-4.4.28-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_armv7a_cros_linux_gnueabihf_linux_headers_4_14_r2",
        downloaded_file_path = "linux-headers-4.14-r2.tbz2",
        sha256 = "876b7a0ee5e9a69261f917e418fbc107139cf29ca907fb7b028403394991d13b",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-armv7a-cros-linux-gnueabihf/linux-headers-4.14-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_armv7a_cros_linux_gnueabihf_llvm_libunwind_17_0_pre496208_r7",
        downloaded_file_path = "llvm-libunwind-17.0_pre496208-r7.tbz2",
        sha256 = "fced5816187dc50d4443951c91186f15ec434d3c9f7c4254300d1f399a8fcab0",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-armv7a-cros-linux-gnueabihf/llvm-libunwind-17.0_pre496208-r7.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_x86_64_cros_linux_gnu_binutils_2_39_r3",
        downloaded_file_path = "binutils-2.39-r3.tbz2",
        sha256 = "b7628003d628f4101090f79c42a61946cf986fa9ad7d5be3b40c41acf88013ba",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-x86_64-cros-linux-gnu/binutils-2.39-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_x86_64_cros_linux_gnu_gcc_10_2_0_r34",
        downloaded_file_path = "gcc-10.2.0-r34.tbz2",
        sha256 = "c8c4ecf0f0b4a73e8baea332f9ceab0701da5f2e21d13ab7ec64752eff3a014e",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-x86_64-cros-linux-gnu/gcc-10.2.0-r34.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_x86_64_cros_linux_gnu_gdb_9_2_20200923_r10",
        downloaded_file_path = "gdb-9.2.20200923-r10.tbz2",
        sha256 = "437881569875203e3b8cf4351abb2c667e001338f20f2e381377d5d66e6a1ccf",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-x86_64-cros-linux-gnu/gdb-9.2.20200923-r10.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_x86_64_cros_linux_gnu_glibc_2_35_r22",
        downloaded_file_path = "glibc-2.35-r22.tbz2",
        sha256 = "51750c3ef5dc4170a55cf864dfbefccb19dd9df3dac0eedc5112ec5a57bcf98f",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-x86_64-cros-linux-gnu/glibc-2.35-r22.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_x86_64_cros_linux_gnu_go_1_20_5_r1",
        downloaded_file_path = "go-1.20.5-r1.tbz2",
        sha256 = "d944b40df4164bbc2acaf49a655cb51159767331d66b7558a2a63e1e01014eeb",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-x86_64-cros-linux-gnu/go-1.20.5-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_x86_64_cros_linux_gnu_libcxx_9999",
        downloaded_file_path = "libcxx-9999.tbz2",
        sha256 = "0f5ad6f559ef4facf7c367a02f55c52836185a77493ba033c9c91cafbfa76b4a",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-x86_64-cros-linux-gnu/libcxx-9999.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_x86_64_cros_linux_gnu_libxcrypt_4_4_28_r2",
        downloaded_file_path = "libxcrypt-4.4.28-r2.tbz2",
        sha256 = "5158869e3b0c442f2aa4982a79d60f58e4612a8880669d84ffa9228945e31964",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-x86_64-cros-linux-gnu/libxcrypt-4.4.28-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_x86_64_cros_linux_gnu_linux_headers_4_14_r2",
        downloaded_file_path = "linux-headers-4.14-r2.tbz2",
        sha256 = "9852415448b99bbab9f70d0166757a1d44ed6ba8e8a32882e4fc745abbb6a448",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-x86_64-cros-linux-gnu/linux-headers-4.14-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_cross_x86_64_cros_linux_gnu_llvm_libunwind_17_0_pre496208_r7",
        downloaded_file_path = "llvm-libunwind-17.0_pre496208-r7.tbz2",
        sha256 = "d5a3672dac7c46af16a59a32937b5a77b9b7c8781c531cae9ded4b676d4f5c7f",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/cross-x86_64-cros-linux-gnu/llvm-libunwind-17.0_pre496208-r7.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_dev_embedded_coreboot_sdk_0_0_1_r120",
        downloaded_file_path = "coreboot-sdk-0.0.1-r120.tbz2",
        sha256 = "fb7e0f71b572a11cb1998f1842fb16f661faecc37cc14382bab1329bb2ce4268",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/dev-embedded/coreboot-sdk-0.0.1-r120.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_dev_embedded_hps_sdk_0_0_1_r7",
        downloaded_file_path = "hps-sdk-0.0.1-r7.tbz2",
        sha256 = "f1eba78a6bfd725ee4f71604201d5bb067c6726c47e537190993c9dadeb40f25",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/dev-embedded/hps-sdk-0.0.1-r7.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_dev_lang_rust_1_70_0",
        downloaded_file_path = "rust-1.70.0.tbz2",
        sha256 = "8c4f169cb11ea9bb407266b9c7a6a27f952962543ef361466923b816d7feb6cc",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/dev-lang/rust-1.70.0.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_dev_lang_rust_bootstrap_1_69_0",
        downloaded_file_path = "rust-bootstrap-1.69.0.tbz2",
        sha256 = "21e05005c2bb9f1cd5737e6bdc0b0989bbc53006731c2e9bf1bf20c4e561602c",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/dev-lang/rust-bootstrap-1.69.0.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_dev_lang_rust_host_1_70_0",
        downloaded_file_path = "rust-host-1.70.0.tbz2",
        sha256 = "e5d73e3a5c781132f9b7bd00fdfd0ca31d6eb074e105e7966d9e563fab4f95cc",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/dev-lang/rust-host-1.70.0.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_08_03_170038_dev_util_glib_utils_2_74_1",
        downloaded_file_path = "glib-utils-2.74.1.tbz2",
        sha256 = "a11c84caf26dec82e57ea2906e7db92af1e9ff46b2736d447067de97c4bb2781",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.08.03.170038/packages/dev-util/glib-utils-2.74.1.tbz2"],
    )

    # Use GN binary built without rpmalloc to avoid crash bug.
    # TODO(b/273830995): Remove this.
    http_file(
        name = "gn_without_rpmalloc",
        downloaded_file_path = "gn_without_rpmalloc",
        sha256 = "46ae28050ac648738a908284807184eb3dddd176b7a8054db63c14e8757d5b81",
        urls = ["https://commondatastorage.googleapis.com/chromeos-throw-away-bucket/cros-bazel/gn_without_rpmalloc"],
    )
