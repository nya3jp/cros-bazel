# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
load("@bazel_tools//tools/build_defs/repo:http.bzl", _http_file = "http_file")

def prebuilts_dependencies(http_file = _http_file):
    # portage/tools/sdk_repos.sh 2023.09.08.050046
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_aarch64_cros_linux_gnu_binutils_2_39_r3",
        downloaded_file_path = "binutils-2.39-r3.tbz2",
        sha256 = "e2bfeba2e471ede497e8d3b395f524a1f3ee6490e7bab1308ab1f535255ecfc3",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-aarch64-cros-linux-gnu/binutils-2.39-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_aarch64_cros_linux_gnu_compiler_rt_17_0_pre496208_r8",
        downloaded_file_path = "compiler-rt-17.0_pre496208-r8.tbz2",
        sha256 = "6f5c42a4fe1d5227c5c6169c85fc5a97eb09562f5bec3e1ede085c3d3564b027",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-aarch64-cros-linux-gnu/compiler-rt-17.0_pre496208-r8.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_aarch64_cros_linux_gnu_gcc_10_2_0_r36",
        downloaded_file_path = "gcc-10.2.0-r36.tbz2",
        sha256 = "e3bc5a8b5ccfceeb0c923610c7fa278e80393ef2425af87726ab243cf76cd6d2",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-aarch64-cros-linux-gnu/gcc-10.2.0-r36.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_aarch64_cros_linux_gnu_gdb_9_2_20200923_r12",
        downloaded_file_path = "gdb-9.2.20200923-r12.tbz2",
        sha256 = "b4ca88d3694a84c88fb8020cbff1b07fc2115dd4f09e0eea3efedf08427039c5",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-aarch64-cros-linux-gnu/gdb-9.2.20200923-r12.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_aarch64_cros_linux_gnu_glibc_2_35_r22",
        downloaded_file_path = "glibc-2.35-r22.tbz2",
        sha256 = "2c198d8ea7589eb7d5ba25030831f16fda85f28b2a2e237e31b251383fed9fcf",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-aarch64-cros-linux-gnu/glibc-2.35-r22.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_aarch64_cros_linux_gnu_go_1_20_5_r1",
        downloaded_file_path = "go-1.20.5-r1.tbz2",
        sha256 = "8d93257967def10d12c137371998f720390913212687979c362b906a29cd1071",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-aarch64-cros-linux-gnu/go-1.20.5-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_aarch64_cros_linux_gnu_libcxx_17_0_pre496208_r17",
        downloaded_file_path = "libcxx-17.0_pre496208-r17.tbz2",
        sha256 = "f824f0433956d0ac99494fbdc1d00b955b6409ddc3fef134f95fc97fc64b9f12",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-aarch64-cros-linux-gnu/libcxx-17.0_pre496208-r17.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_aarch64_cros_linux_gnu_libxcrypt_4_4_28_r2",
        downloaded_file_path = "libxcrypt-4.4.28-r2.tbz2",
        sha256 = "f0342c447081e127044294eaf9eb11b2132a2dbeaa0dde173bf8566b9be920f7",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-aarch64-cros-linux-gnu/libxcrypt-4.4.28-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_aarch64_cros_linux_gnu_linux_headers_4_14_r90",
        downloaded_file_path = "linux-headers-4.14-r90.tbz2",
        sha256 = "6008a173c80f24ca00748d2fda67fa633f283a19ac1d8a1bd619940cee8f54c2",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-aarch64-cros-linux-gnu/linux-headers-4.14-r90.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_aarch64_cros_linux_gnu_llvm_libunwind_17_0_pre496208_r12",
        downloaded_file_path = "llvm-libunwind-17.0_pre496208-r12.tbz2",
        sha256 = "6dd831da0dcd9b274c91e87e6ba58bec81869e027855ba305fa6fee8f91d35b6",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-aarch64-cros-linux-gnu/llvm-libunwind-17.0_pre496208-r12.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_armv7a_cros_linux_gnueabihf_binutils_2_39_r3",
        downloaded_file_path = "binutils-2.39-r3.tbz2",
        sha256 = "df71a01c9811104cd4e811cd113b6f9103ebc0e68374c4957e0d8d6d847a7ea7",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-armv7a-cros-linux-gnueabihf/binutils-2.39-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_armv7a_cros_linux_gnueabihf_compiler_rt_17_0_pre496208_r8",
        downloaded_file_path = "compiler-rt-17.0_pre496208-r8.tbz2",
        sha256 = "fc97dc9d14270e89e73f1badc7bfd01170019dc4c780ac726dea8fcc1ca9c80e",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-armv7a-cros-linux-gnueabihf/compiler-rt-17.0_pre496208-r8.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_armv7a_cros_linux_gnueabihf_gcc_10_2_0_r36",
        downloaded_file_path = "gcc-10.2.0-r36.tbz2",
        sha256 = "85ccd6b13e2f4799321bc322a2f1dda314092066e49885e5456b97cd22aae64d",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-armv7a-cros-linux-gnueabihf/gcc-10.2.0-r36.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_armv7a_cros_linux_gnueabihf_gdb_9_2_20200923_r12",
        downloaded_file_path = "gdb-9.2.20200923-r12.tbz2",
        sha256 = "b1934b203d1967c27739f3e337caa317435f59bcd75f24f0dc80e56ebbb28281",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-armv7a-cros-linux-gnueabihf/gdb-9.2.20200923-r12.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_armv7a_cros_linux_gnueabihf_glibc_2_35_r22",
        downloaded_file_path = "glibc-2.35-r22.tbz2",
        sha256 = "ae9995628e4d35d695f1516f3187f0bf5567dba24644826dbab34f25687dc4f7",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-armv7a-cros-linux-gnueabihf/glibc-2.35-r22.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_armv7a_cros_linux_gnueabihf_go_1_20_5_r1",
        downloaded_file_path = "go-1.20.5-r1.tbz2",
        sha256 = "7ca57627e58fd0d5e649b53d8d193b6e92042896e13e10702105d4c3a1c9dab8",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-armv7a-cros-linux-gnueabihf/go-1.20.5-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_armv7a_cros_linux_gnueabihf_libcxx_17_0_pre496208_r17",
        downloaded_file_path = "libcxx-17.0_pre496208-r17.tbz2",
        sha256 = "67d93c1647248fb90fcf31d304774272b35cb917561559e73bddceb81453fe09",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-armv7a-cros-linux-gnueabihf/libcxx-17.0_pre496208-r17.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_armv7a_cros_linux_gnueabihf_libxcrypt_4_4_28_r2",
        downloaded_file_path = "libxcrypt-4.4.28-r2.tbz2",
        sha256 = "86c7d7f4c4d20ef6f68e98a1d770bb205f620de21c2b625fbe4110bb9580d42d",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-armv7a-cros-linux-gnueabihf/libxcrypt-4.4.28-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_armv7a_cros_linux_gnueabihf_linux_headers_4_14_r90",
        downloaded_file_path = "linux-headers-4.14-r90.tbz2",
        sha256 = "747c1bea9da32be31b6a903406c20e59fff9d05c6fd43d71d327ec58ac57f157",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-armv7a-cros-linux-gnueabihf/linux-headers-4.14-r90.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_armv7a_cros_linux_gnueabihf_llvm_libunwind_17_0_pre496208_r12",
        downloaded_file_path = "llvm-libunwind-17.0_pre496208-r12.tbz2",
        sha256 = "5b3f4c931c8e87a64eaf0cde02d8424331bc48026cd4918bbb52313978d64df6",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-armv7a-cros-linux-gnueabihf/llvm-libunwind-17.0_pre496208-r12.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_x86_64_cros_linux_gnu_binutils_2_39_r3",
        downloaded_file_path = "binutils-2.39-r3.tbz2",
        sha256 = "d9e492101894f0fa45a4a4ec2b5f3e28e0056139261da486575d80af6a75bd82",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-x86_64-cros-linux-gnu/binutils-2.39-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_x86_64_cros_linux_gnu_gcc_10_2_0_r36",
        downloaded_file_path = "gcc-10.2.0-r36.tbz2",
        sha256 = "b1ea4f4e759974c127e22b542dddbeb1dcc1851cb7dcef3f38f421785712a31e",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-x86_64-cros-linux-gnu/gcc-10.2.0-r36.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_x86_64_cros_linux_gnu_gdb_9_2_20200923_r12",
        downloaded_file_path = "gdb-9.2.20200923-r12.tbz2",
        sha256 = "c61bd1de5dab5c91767404f65b7f043ca2fd79379c202c9339c8a91f2aa0d152",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-x86_64-cros-linux-gnu/gdb-9.2.20200923-r12.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_x86_64_cros_linux_gnu_glibc_2_35_r22",
        downloaded_file_path = "glibc-2.35-r22.tbz2",
        sha256 = "5d4bec3e3e7f4782b7739bba936f082c0a33f458b44e374a50bdfdf7fdd97d4a",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-x86_64-cros-linux-gnu/glibc-2.35-r22.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_x86_64_cros_linux_gnu_go_1_20_5_r1",
        downloaded_file_path = "go-1.20.5-r1.tbz2",
        sha256 = "55cab78ecba765e7a0f1a03d76361a163e26f8c61bb34aadf05d800b7f1df283",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-x86_64-cros-linux-gnu/go-1.20.5-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_x86_64_cros_linux_gnu_libcxx_17_0_pre496208_r17",
        downloaded_file_path = "libcxx-17.0_pre496208-r17.tbz2",
        sha256 = "95b3516c86bfb6bebbeae6f9198385604f2c4b6bff8d77cc4a0b3218cd6d0217",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-x86_64-cros-linux-gnu/libcxx-17.0_pre496208-r17.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_x86_64_cros_linux_gnu_libxcrypt_4_4_28_r2",
        downloaded_file_path = "libxcrypt-4.4.28-r2.tbz2",
        sha256 = "45d504138324fe232cbc97bcf15689484b289d28118f22ede17f7b89b2c67949",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-x86_64-cros-linux-gnu/libxcrypt-4.4.28-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_x86_64_cros_linux_gnu_linux_headers_4_14_r90",
        downloaded_file_path = "linux-headers-4.14-r90.tbz2",
        sha256 = "1da42b7c0bf436886bf149ec142bf68cbd56d6763a8e2986b72000961ad169dd",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-x86_64-cros-linux-gnu/linux-headers-4.14-r90.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_cross_x86_64_cros_linux_gnu_llvm_libunwind_17_0_pre496208_r12",
        downloaded_file_path = "llvm-libunwind-17.0_pre496208-r12.tbz2",
        sha256 = "340957a9a84e8bdc1771b280976c58a536e822b83497f14b8ad329a34ff539c8",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/cross-x86_64-cros-linux-gnu/llvm-libunwind-17.0_pre496208-r12.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_dev_embedded_coreboot_sdk_0_0_1_r120",
        downloaded_file_path = "coreboot-sdk-0.0.1-r120.tbz2",
        sha256 = "798f3c452d78929b57bb026b7d793f8d7865fee105ce186e21a6ca969f07d6e7",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/dev-embedded/coreboot-sdk-0.0.1-r120.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_dev_embedded_hps_sdk_0_0_1_r8",
        downloaded_file_path = "hps-sdk-0.0.1-r8.tbz2",
        sha256 = "9783f09b807e241bd4576cf52a6f4cdb4e05cab2c5c17a43fa5fd641dc7a7000",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/dev-embedded/hps-sdk-0.0.1-r8.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_dev_lang_rust_1_70_0",
        downloaded_file_path = "rust-1.70.0.tbz2",
        sha256 = "09db67394d4d46774c933c344f788b7fb23e5e15aa51f154e0820cf70bb4a953",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/dev-lang/rust-1.70.0.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_dev_lang_rust_bootstrap_1_69_0",
        downloaded_file_path = "rust-bootstrap-1.69.0.tbz2",
        sha256 = "60439ad65c353ff12e15e5bc17d708467f5b8fdbc1d0cb72fa2cca268b2ed79c",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/dev-lang/rust-bootstrap-1.69.0.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_dev_lang_rust_host_1_70_0",
        downloaded_file_path = "rust-host-1.70.0.tbz2",
        sha256 = "864f8cd6b625d4ceae31247dac3a324825133ce29a14e69d669f23c4e55af5ad",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/dev-lang/rust-host-1.70.0.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_09_08_050046_dev_util_glib_utils_2_74_1",
        downloaded_file_path = "glib-utils-2.74.1.tbz2",
        sha256 = "0c087edf76987ddcd7377656a7d30425324c81f2f9730da2711ef890c52ab964",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.08.050046/packages/dev-util/glib-utils-2.74.1.tbz2"],
    )
