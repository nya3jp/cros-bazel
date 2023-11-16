# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
load("@bazel_tools//tools/build_defs/repo:http.bzl", _http_file = "http_file")

def prebuilts_dependencies(http_file = _http_file):
    # portage/tools/sdk_repos.sh 2023.11.15.020032
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_binutils",
        downloaded_file_path = "binutils-2.40.tbz2",
        sha256 = "86ee3cc3499e097919c143c31c18c9b81cf858660fb763cf72a5252dff1d1e59",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-aarch64-cros-linux-gnu/binutils-2.40.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_compiler_rt",
        downloaded_file_path = "compiler-rt-18.0_pre510928-r17.tbz2",
        sha256 = "8b10063577df207c8400e848ed165f8b5de59c6d76d432b99292aaadc7f345da",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-aarch64-cros-linux-gnu/compiler-rt-18.0_pre510928-r17.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_gcc",
        downloaded_file_path = "gcc-10.2.0-r39.tbz2",
        sha256 = "10fbecddea56ef68daf18a78bc992385f8db981eb74cb7e048685c04f9192e60",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-aarch64-cros-linux-gnu/gcc-10.2.0-r39.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_gdb",
        downloaded_file_path = "gdb-9.2.20200923-r12.tbz2",
        sha256 = "92ba9c9c5c39af74bd2c2ca5489f55f59bdd852e56fff0ebcffe6e93669de299",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-aarch64-cros-linux-gnu/gdb-9.2.20200923-r12.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_glibc",
        downloaded_file_path = "glibc-2.35-r25.tbz2",
        sha256 = "5ce7b9779c1e8a97cf945e664a9ea0cc1ea51f12fa91fb8423e1d0205ae67c22",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-aarch64-cros-linux-gnu/glibc-2.35-r25.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_go",
        downloaded_file_path = "go-1.20.5-r1.tbz2",
        sha256 = "b74d3e5e8f1a8469d8968a735afffeeba362fb503db57ec619a1f59cf17ada4e",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-aarch64-cros-linux-gnu/go-1.20.5-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_libcxx",
        downloaded_file_path = "libcxx-18.0_pre510928-r27.tbz2",
        sha256 = "d38482fb950c874c6755d812beff9d9afd1b47aae31a72784f9806ba066da02d",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-aarch64-cros-linux-gnu/libcxx-18.0_pre510928-r27.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_libxcrypt",
        downloaded_file_path = "libxcrypt-4.4.28-r3.tbz2",
        sha256 = "a6d35fa96e6058035a73284c0a4ca83f40c57d53da7c5c1edcc5af4346179844",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-aarch64-cros-linux-gnu/libxcrypt-4.4.28-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_linux_headers",
        downloaded_file_path = "linux-headers-4.14-r91.tbz2",
        sha256 = "84933dffca9d263651473b080ec279858cfa02ed7c168031950d8e7f1461a1be",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-aarch64-cros-linux-gnu/linux-headers-4.14-r91.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_llvm_libunwind",
        downloaded_file_path = "llvm-libunwind-18.0_pre510928-r21.tbz2",
        sha256 = "0c337a33f32a02faec5fcb75f8d7bbe2c4012df86d657880fc81543dba6be875",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-aarch64-cros-linux-gnu/llvm-libunwind-18.0_pre510928-r21.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_binutils",
        downloaded_file_path = "binutils-2.40.tbz2",
        sha256 = "1b8398237ad4539d3a2deb9e7b9c0b81015e5bf2f9810790d28e512b2e01ea45",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-armv7a-cros-linux-gnueabihf/binutils-2.40.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_compiler_rt",
        downloaded_file_path = "compiler-rt-18.0_pre510928-r17.tbz2",
        sha256 = "9595293ae99eec6f96dd92ba70d116071ac45be72f1cc1254b0481c299148285",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-armv7a-cros-linux-gnueabihf/compiler-rt-18.0_pre510928-r17.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_gcc",
        downloaded_file_path = "gcc-10.2.0-r39.tbz2",
        sha256 = "174b59e22482faabda42cdbbbbb300b481b2e4c67efbad1ba1f445dbb19ad309",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-armv7a-cros-linux-gnueabihf/gcc-10.2.0-r39.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_gdb",
        downloaded_file_path = "gdb-9.2.20200923-r12.tbz2",
        sha256 = "dc348ded66ccf7b8c5a30158bf44266f9bb6ec062bc8ebaf8d581aed0abe7848",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-armv7a-cros-linux-gnueabihf/gdb-9.2.20200923-r12.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_glibc",
        downloaded_file_path = "glibc-2.35-r25.tbz2",
        sha256 = "d4527672dbd7beb476640e9b08fa9009e0132f23a025c03c10872a66d39bf3ab",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-armv7a-cros-linux-gnueabihf/glibc-2.35-r25.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_go",
        downloaded_file_path = "go-1.20.5-r1.tbz2",
        sha256 = "7e364c57bb3c8f1bc4f99df017ae5f6a0f78a84018b2f9cae043d49251536724",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-armv7a-cros-linux-gnueabihf/go-1.20.5-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_libcxx",
        downloaded_file_path = "libcxx-18.0_pre510928-r27.tbz2",
        sha256 = "8cee9b88913cf251c54a9a0c1f8e103ffbf55b52fb3e451ea140b00d656b08d6",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-armv7a-cros-linux-gnueabihf/libcxx-18.0_pre510928-r27.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_libxcrypt",
        downloaded_file_path = "libxcrypt-4.4.28-r3.tbz2",
        sha256 = "02b8d1921795a46184162a2c0fc97ee88262b865235178e12f1288e47f5d9cee",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-armv7a-cros-linux-gnueabihf/libxcrypt-4.4.28-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_linux_headers",
        downloaded_file_path = "linux-headers-4.14-r91.tbz2",
        sha256 = "585f106456d09a08fc517b3f70017557ece4fe51d1fdd5efe316040f38923dc7",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-armv7a-cros-linux-gnueabihf/linux-headers-4.14-r91.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_llvm_libunwind",
        downloaded_file_path = "llvm-libunwind-18.0_pre510928-r21.tbz2",
        sha256 = "fa5cdd46b2173198132eb124ed284a8d094b92c05a864dca8766af792da324cd",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-armv7a-cros-linux-gnueabihf/llvm-libunwind-18.0_pre510928-r21.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_binutils",
        downloaded_file_path = "binutils-2.40.tbz2",
        sha256 = "26cf24169b451ee4761d3a6ea32ec3c9800d60aa1bb0fa182720066152ab40dd",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-x86_64-cros-linux-gnu/binutils-2.40.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_gcc",
        downloaded_file_path = "gcc-10.2.0-r39.tbz2",
        sha256 = "05ee2875d5357e5d2054fa32096afb51d62ee8812664cadb4b012c5a4797d13f",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-x86_64-cros-linux-gnu/gcc-10.2.0-r39.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_gdb",
        downloaded_file_path = "gdb-9.2.20200923-r12.tbz2",
        sha256 = "e923593c1329650dc4acc50c5e5c8fd1c5d0f595e96d93fb381a25cee56de3f1",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-x86_64-cros-linux-gnu/gdb-9.2.20200923-r12.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_glibc",
        downloaded_file_path = "glibc-2.35-r25.tbz2",
        sha256 = "1349bfaed39d897d11f71da0bd100bf5e68b3c478d6a9fac908df0fd3f839804",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-x86_64-cros-linux-gnu/glibc-2.35-r25.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_go",
        downloaded_file_path = "go-1.20.5-r1.tbz2",
        sha256 = "0ed38ea5c77305c959c054acd610e359165ea560a0be6e9a06606bab76ddef74",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-x86_64-cros-linux-gnu/go-1.20.5-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_libcxx",
        downloaded_file_path = "libcxx-18.0_pre510928-r27.tbz2",
        sha256 = "3e5e9c7169ab374f141b35fa45802a443b56ae6b809ee4c6222d60c1eceb9391",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-x86_64-cros-linux-gnu/libcxx-18.0_pre510928-r27.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_libxcrypt",
        downloaded_file_path = "libxcrypt-4.4.28-r3.tbz2",
        sha256 = "2564948bbb1bfaad3aa94c07f9cf64acf44843e8b32617b52ac7b6b7b93f70d3",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-x86_64-cros-linux-gnu/libxcrypt-4.4.28-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_linux_headers",
        downloaded_file_path = "linux-headers-4.14-r91.tbz2",
        sha256 = "a79107d044fbcbe03d8d9020502f442bb758872a6bc35526d0a0803a1607e319",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-x86_64-cros-linux-gnu/linux-headers-4.14-r91.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_llvm_libunwind",
        downloaded_file_path = "llvm-libunwind-18.0_pre510928-r21.tbz2",
        sha256 = "f41346de085c4888f6b72845ff4eda3b1b40b7c522f963cba78aedd4163dcc9a",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/cross-x86_64-cros-linux-gnu/llvm-libunwind-18.0_pre510928-r21.tbz2"],
    )
    http_file(
        name = "amd64_host_dev_embedded_coreboot_sdk",
        downloaded_file_path = "coreboot-sdk-0.0.1-r121.tbz2",
        sha256 = "09e1ed7ba9b0d8bcf84ae410026106798352a32fe5556d1ee8ab540446bd1333",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/dev-embedded/coreboot-sdk-0.0.1-r121.tbz2"],
    )
    http_file(
        name = "amd64_host_dev_embedded_hps_sdk",
        downloaded_file_path = "hps-sdk-0.0.1-r9.tbz2",
        sha256 = "c041a816117293e276da8193ddcda6f5d0af337468a298d6fc7bfbe60101fe22",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/dev-embedded/hps-sdk-0.0.1-r9.tbz2"],
    )
    http_file(
        name = "amd64_host_dev_lang_rust",
        downloaded_file_path = "rust-1.72.1.tbz2",
        sha256 = "782ca54b45bd04f2c9f73044d3a2328acb76407d332e53e0b1b73f1818be783b",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/dev-lang/rust-1.72.1.tbz2"],
    )
    http_file(
        name = "amd64_host_dev_util_glib_utils",
        downloaded_file_path = "glib-utils-2.76.4.tbz2",
        sha256 = "c7a85718a4ff48afcb5bb1d9100f50151368fb88ae34e9d6e121b1e0fb02d7f3",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.11.15.020032/packages/dev-util/glib-utils-2.76.4.tbz2"],
    )
