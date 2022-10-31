# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")

def prebuilts_dependencies():
    http_file(
        name = "arm64_generic_linux_headers_4_14_r56",
        downloaded_file_path = "linux-headers-4.14-r56.tbz2",
        sha256 = "267cd8b40682a42079b9942a78b533a9bcfffffa1229cff795f3738010fce140",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/board/arm64-generic/postsubmit-R107-15066.0.0-38990-8804972997102079969/packages/sys-kernel/linux-headers-4.14-r56.tbz2"],
    )
    http_file(
        name = "arm64_generic_gcc_libs_10_2_0_r4",
        downloaded_file_path = "gcc-libs-10.2.0-r4.tbz2",
        sha256 = "750e4c73763d1d60bc69d90dc76d5f8096332757072fafd0fd30865f17708390",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/board/arm64-generic/postsubmit-R107-15066.0.0-38990-8804972997102079969/packages/sys-libs/gcc-libs-10.2.0-r4.tbz2"],
    )
    http_file(
        name = "arm64_generic_libcxx_15_0_pre458507_r6",
        downloaded_file_path = "libcxx-15.0_pre458507-r6.tbz2",
        sha256 = "538878dd3557fb041749254196a58775c1e1b3dec321b6f0729df655750701ae",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/board/arm64-generic/postsubmit-R107-15066.0.0-38990-8804972997102079969/packages/sys-libs/libcxx-15.0_pre458507-r6.tbz2"],
    )
    http_file(
        name = "arm64_generic_llvm_libunwind_15_0_pre458507_r4",
        downloaded_file_path = "llvm-libunwind-15.0_pre458507-r4.tbz2",
        sha256 = "ffdac376ff4b1989cc97fcec9d7ff21dc30e352877b41c16e78958ec21df26ad",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/board/arm64-generic/postsubmit-R107-15066.0.0-38990-8804972997102079969/packages/sys-libs/llvm-libunwind-15.0_pre458507-r4.tbz2"],
    )
    http_file(
        name = "arm64_generic_chromeos_fonts_0_0_1_r52",
        downloaded_file_path = "chromeos-fonts-0.0.1-r52.tbz2",
        sha256 = "40128c1465aa6ca717561be8acda996478cc41aa414fda18d64cbaa37509c02c",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/board/arm64-generic/postsubmit-R107-15066.0.0-38990-8804972997102079969/packages/chromeos-base/chromeos-fonts-0.0.1-r52.tbz2"],
    )
    http_file(
        name = "arm64_generic_chromeos_icu_107_0_5257_r1",
        downloaded_file_path = "chrome-icu-107.0.5257.0_rc-r1.tbz2",
        sha256 = "a11edea1e2ba319559921a225e4949a54267e15f10527aecc11f79155d10077e",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/board/arm64-generic/postsubmit-R107-15066.0.0-38990-8804972997102079969/packages/chromeos-base/chrome-icu-107.0.5257.0_rc-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_binutils_2_36_1_r8",
        downloaded_file_path = "binutils-2.36.1-r8.tbz2",
        sha256 = "c790efb90da825d0c169c34e191290826e03e21031c6c993e1d337b8b9c7d042",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/binutils-2.36.1-r8.tbz2"],
    )
    http_file(
        name = "amd64_host_compiler_rt_15_0_pre458507_r6",
        downloaded_file_path = "compiler-rt-15.0_pre458507-r6.tbz2",
        sha256 = "39723dbd256b02ec19b9293a18139a12d6764c083380a90bdb28e13471690727",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/compiler-rt-15.0_pre458507-r6.tbz2"],
    )
    http_file(
        name = "amd64_host_gcc_10_2_0_r28",
        downloaded_file_path = "gcc-10.2.0-r28.tbz2",
        sha256 = "bca12617716fc725143a34894b9eb45116061e6e37fbd4a3fcf28bb67660b395",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/gcc-10.2.0-r28.tbz2"],
    )
    http_file(
        name = "amd64_host_gdb_9_2_20200923_r9",
        downloaded_file_path = "gdb-9.2.20200923-r9.tbz2",
        sha256 = "ec4d7d2a0bf54872fc7de56777f6d80773a88dada52eb88cb3b2eccf88ac9dac",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/gdb-9.2.20200923-r9.tbz2"],
    )
    http_file(
        name = "amd64_host_glibc_2_33_r17",
        downloaded_file_path = "glibc-2.33-r17.tbz2",
        sha256 = "8a21f6c510bdbead7d86351a24cd5362272cb53044be13e0a1d482c66c24f1ae",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/glibc-2.33-r17.tbz2"],
    )
    http_file(
        name = "amd64_host_go_1_18_r2",
        downloaded_file_path = "go-1.18-r2.tbz2",
        sha256 = "3b3d0066a46a7cc535eaf60a1c23aeaca7b4ff3b6edd565c6f2f31ef1b470ba0",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/go-1.18-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_libcxx_15_0_pre458507_r6",
        downloaded_file_path = "libcxx-15.0_pre458507-r6.tbz2",
        sha256 = "795a65043849b065741a8cc50119b5f331d3406cb84263b975a370186e0b7344",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/libcxx-15.0_pre458507-r6.tbz2"],
    )
    http_file(
        name = "amd64_host_libxcrypt_4_4_28_r1",
        downloaded_file_path = "libxcrypt-4.4.28-r1.tbz2",
        sha256 = "a925eed789030a8431084abb8b6e1c985ec0179e26fd83f266c4fa183f135b41",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/libxcrypt-4.4.28-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_linux_headers_4_14_r56",
        downloaded_file_path = "linux-headers-4.14-r56.tbz2",
        sha256 = "b79b881f88ff8c639dfa0a012aeda61f6041c169d946fc5071298b8cd23ea597",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/linux-headers-4.14-r56.tbz2"],
    )
    http_file(
        name = "amd64_host_llvm_libunwind_15_0_pre458507_r4",
        downloaded_file_path = "llvm-libunwind-15.0_pre458507-r4.tbz2",
        sha256 = "af0295d8ce5d8c3621864c8a328193d493afa3984ccebe8fd6afa3f46bc6b855",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/llvm-libunwind-15.0_pre458507-r4.tbz2"],
    )
    http_file(
        name = "amd64_host_docbook_xml_dtd_4_4_r3",
        downloaded_file_path = "docbook-xml-dtd-4.4-r3.tbz2",
        sha256 = "9a5e7219710bddfb977bbc419a7df2dec71000c909c69484be13a8d3a89e5232",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/app-text/docbook-xml-dtd-4.4-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_meson_format_array_0",
        downloaded_file_path = "meson-format-array-0.tbz2",
        sha256 = "09d863e180a251b745a91e2c4d4d39ddf7f97a89bd851bc8ec460f041f387ffb",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/dev-util/meson-format-array-0.tbz2"],
    )
    http_file(
        name = "amd64_host_rust_1_62_1",
        downloaded_file_path = "rust-1.62.1.tbz2",
        sha256 = "03af218671910e7f590fe92a0f3ba7d4c076c1390f9f1c9cd2d71b85f50a4744",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/dev-lang/rust-1.62.1.tbz2"],
    )
    http_file(
        name = "amd64_host_hps_sdk_0_0_0_r4",
        downloaded_file_path = "hps-sdk-0.0.1-r4.tbz2",
        sha256 = "3b96735cadcfb285f2f2c740e0c6bb595be60853b57649d15e4c815f72fdef7c",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/dev-embedded/hps-sdk-0.0.1-r4.tbz2"],
    )
    http_file(
        name = "amd64_host_xcb_proto_1_14_1",
        downloaded_file_path = "xcb-proto-1.14.1.tbz2",
        sha256 = "848f74ec91f249c11ca462729ede8136190e8fbdce647782c8d0b2fd2531a2f9",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/x11-base/xcb-proto-1.14.1.tbz2"],
    )
