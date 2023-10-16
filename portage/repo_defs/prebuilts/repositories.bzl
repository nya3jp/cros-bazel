# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
load("@bazel_tools//tools/build_defs/repo:http.bzl", _http_file = "http_file")

def prebuilts_dependencies(http_file = _http_file):
    # portage/tools/sdk_repos.sh 2023.10.03.020014
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_binutils",
        downloaded_file_path = "binutils-2.39-r3.tbz2",
        sha256 = "c6b645823a43dddbb283e6a6a82f93b8079a6e7292626fe2f2adcca1a4bc592e",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-aarch64-cros-linux-gnu/binutils-2.39-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_compiler_rt",
        downloaded_file_path = "compiler-rt-17.0_pre498229-r9.tbz2",
        sha256 = "bc464fe96c99835009aa16dcb6e92dec8cf134f89984a78e52cb2a716393eb30",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-aarch64-cros-linux-gnu/compiler-rt-17.0_pre498229-r9.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_gcc",
        downloaded_file_path = "gcc-10.2.0-r37.tbz2",
        sha256 = "404e15f08cf4fa8705f5154ddc42aa995e83c78cdd4a02646d1f3e8c4a5e4dbc",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-aarch64-cros-linux-gnu/gcc-10.2.0-r37.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_gdb",
        downloaded_file_path = "gdb-9.2.20200923-r12.tbz2",
        sha256 = "6d53c5fe3e99990a0263928262d887d47f6f3bd4b565008648d6d5f9b919f3ac",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-aarch64-cros-linux-gnu/gdb-9.2.20200923-r12.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_glibc",
        downloaded_file_path = "glibc-2.35-r24.tbz2",
        sha256 = "b798996081bc646063edde84dd12eedd8fbfedf8cfc43d8e4b746f178eaeff0e",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-aarch64-cros-linux-gnu/glibc-2.35-r24.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_go",
        downloaded_file_path = "go-1.20.5-r1.tbz2",
        sha256 = "c71298cd9bc71958e9799efb4e734be0d77aa080307d9aa347af7bae82d342ac",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-aarch64-cros-linux-gnu/go-1.20.5-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_libcxx",
        downloaded_file_path = "libcxx-17.0_pre498229-r18.tbz2",
        sha256 = "4a277cc2dd9412907a7ed4e386f7e8189e41f49a76a193a22c02d9f43922c5ad",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-aarch64-cros-linux-gnu/libcxx-17.0_pre498229-r18.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_libxcrypt",
        downloaded_file_path = "libxcrypt-4.4.28-r3.tbz2",
        sha256 = "3d9445b54fe1cc1841a4a43796410b886f24ac36e84f79cb893dd77091710006",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-aarch64-cros-linux-gnu/libxcrypt-4.4.28-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_linux_headers",
        downloaded_file_path = "linux-headers-4.14-r91.tbz2",
        sha256 = "9e4ac3e2bfbde9ae8ab202181a0499ce2487463081856aa451b1e47bbff6b64b",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-aarch64-cros-linux-gnu/linux-headers-4.14-r91.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_llvm_libunwind",
        downloaded_file_path = "llvm-libunwind-17.0_pre498229-r14.tbz2",
        sha256 = "6f19180e276d2f43345f96c186360f5b4c66108d3e7952269ecc99774a828448",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-aarch64-cros-linux-gnu/llvm-libunwind-17.0_pre498229-r14.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_binutils",
        downloaded_file_path = "binutils-2.39-r3.tbz2",
        sha256 = "e14b2966e170e59acc8db5f302acc1a4888f843d3e32919977cf5664d0108430",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-armv7a-cros-linux-gnueabihf/binutils-2.39-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_compiler_rt",
        downloaded_file_path = "compiler-rt-17.0_pre498229-r9.tbz2",
        sha256 = "d1e5b09dd916676f9109b3b5c65f84e181979e545c6276d33744e20a636d83c8",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-armv7a-cros-linux-gnueabihf/compiler-rt-17.0_pre498229-r9.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_gcc",
        downloaded_file_path = "gcc-10.2.0-r37.tbz2",
        sha256 = "e7ba2d7a76804352fa719fca5d3f01925ad1f216bb58cfbcf669419ee296404e",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-armv7a-cros-linux-gnueabihf/gcc-10.2.0-r37.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_gdb",
        downloaded_file_path = "gdb-9.2.20200923-r12.tbz2",
        sha256 = "aaf0fd984326af4457b6140056be56f90770f28dbb5a9d4d843463aaa1919bab",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-armv7a-cros-linux-gnueabihf/gdb-9.2.20200923-r12.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_glibc",
        downloaded_file_path = "glibc-2.35-r24.tbz2",
        sha256 = "3e8a2232ad30e16929f1884239ee6d3522ed188e327d4e67d24600943405c9bb",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-armv7a-cros-linux-gnueabihf/glibc-2.35-r24.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_go",
        downloaded_file_path = "go-1.20.5-r1.tbz2",
        sha256 = "4129b866f5dd560da1f9f2a41e3e90cf01a269352b7b3c7e0b3e596620985511",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-armv7a-cros-linux-gnueabihf/go-1.20.5-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_libcxx",
        downloaded_file_path = "libcxx-17.0_pre498229-r18.tbz2",
        sha256 = "1c192d0037b028d88fa859e222dfb8b44032e850d1861ccd89b3a3cfdee0596b",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-armv7a-cros-linux-gnueabihf/libcxx-17.0_pre498229-r18.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_libxcrypt",
        downloaded_file_path = "libxcrypt-4.4.28-r3.tbz2",
        sha256 = "97e27d249554fb47f84e632584eada46bd279299ae216c3af7dc7ec4657b0ff0",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-armv7a-cros-linux-gnueabihf/libxcrypt-4.4.28-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_linux_headers",
        downloaded_file_path = "linux-headers-4.14-r91.tbz2",
        sha256 = "b8dde55e8965b5d83de1b8fe5fcbd137b132a57b014bc80457467a31ff1e14f3",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-armv7a-cros-linux-gnueabihf/linux-headers-4.14-r91.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_llvm_libunwind",
        downloaded_file_path = "llvm-libunwind-17.0_pre498229-r14.tbz2",
        sha256 = "0009334c09d588990a8a0955ea72714f042afd01ff3ada6751fa69714dc92e54",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-armv7a-cros-linux-gnueabihf/llvm-libunwind-17.0_pre498229-r14.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_binutils",
        downloaded_file_path = "binutils-2.39-r3.tbz2",
        sha256 = "c16efb50a624c875e7e4b5fde0cabc24b04e799ff159de9dbdf3da4a7c58a597",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-x86_64-cros-linux-gnu/binutils-2.39-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_gcc",
        downloaded_file_path = "gcc-10.2.0-r37.tbz2",
        sha256 = "7759769b6c56b645c3b21af0ee86610a4eda22bb0b5e4bf6b7a18d621402847a",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-x86_64-cros-linux-gnu/gcc-10.2.0-r37.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_gdb",
        downloaded_file_path = "gdb-9.2.20200923-r12.tbz2",
        sha256 = "462abbd1498be589ec3f197a6923b24636165c6e3fe68b5e89654828fc8708cd",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-x86_64-cros-linux-gnu/gdb-9.2.20200923-r12.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_glibc",
        downloaded_file_path = "glibc-2.35-r24.tbz2",
        sha256 = "25b4c34ae66128bea45b8e5ddd032e89b868e8898377f65713c28e5899099d0d",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-x86_64-cros-linux-gnu/glibc-2.35-r24.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_go",
        downloaded_file_path = "go-1.20.5-r1.tbz2",
        sha256 = "aaa76c10cf8e807d792930cf7b3177022731edfb162e9fe8a7fec862a307badc",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-x86_64-cros-linux-gnu/go-1.20.5-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_libcxx",
        downloaded_file_path = "libcxx-17.0_pre498229-r18.tbz2",
        sha256 = "7327315e9103465adda56273fea31f83e86c29f1f603423a262f3841e246004e",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-x86_64-cros-linux-gnu/libcxx-17.0_pre498229-r18.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_libxcrypt",
        downloaded_file_path = "libxcrypt-4.4.28-r3.tbz2",
        sha256 = "818f40596784a12754f765dbafb223c69dbf7a24d796f63160bfebe910f769f1",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-x86_64-cros-linux-gnu/libxcrypt-4.4.28-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_linux_headers",
        downloaded_file_path = "linux-headers-4.14-r91.tbz2",
        sha256 = "4fbd61a46064ba9ae9491b46441c7380f31f6b622b36717f00fc92a76428a2bc",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-x86_64-cros-linux-gnu/linux-headers-4.14-r91.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_llvm_libunwind",
        downloaded_file_path = "llvm-libunwind-17.0_pre498229-r14.tbz2",
        sha256 = "a7547513c1b3948e1ae2fa867a549b15098633a7bdf6b76e5d107903ce51f143",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/cross-x86_64-cros-linux-gnu/llvm-libunwind-17.0_pre498229-r14.tbz2"],
    )
    http_file(
        name = "amd64_host_dev_embedded_coreboot_sdk",
        downloaded_file_path = "coreboot-sdk-0.0.1-r120.tbz2",
        sha256 = "082483ce2b55f8c38e4d2b16a6797c23b714734c9d8cd4be6c98c132cd25ef1b",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/dev-embedded/coreboot-sdk-0.0.1-r120.tbz2"],
    )
    http_file(
        name = "amd64_host_dev_embedded_hps_sdk",
        downloaded_file_path = "hps-sdk-0.0.1-r9.tbz2",
        sha256 = "7794becf818abff21dcec444421a3b96e4d967ce3813c5e813902895dea2e0bd",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/dev-embedded/hps-sdk-0.0.1-r9.tbz2"],
    )
    http_file(
        name = "amd64_host_dev_lang_rust",
        downloaded_file_path = "rust-1.71.1.tbz2",
        sha256 = "696ea519a175798f270a01667df178b5f05e4418f4f1ea62a0e4115659c8a805",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/dev-lang/rust-1.71.1.tbz2"],
    )
    http_file(
        name = "amd64_host_dev_util_glib_utils",
        downloaded_file_path = "glib-utils-2.74.1.tbz2",
        sha256 = "07442d7f7656e58bdb3ec335a4380e496c81fa3825924580ff8a40eff5f495df",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.10.03.020014/packages/dev-util/glib-utils-2.74.1.tbz2"],
    )
