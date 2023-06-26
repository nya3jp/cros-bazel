# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
load("@bazel_tools//tools/build_defs/repo:http.bzl", _http_file = "http_file")

def prebuilts_dependencies(http_file = _http_file):
    # TODO: Delete chromeos-fonts
    http_file(
        name = "arm64_generic_chromeos_fonts_0_0_1_r52",
        downloaded_file_path = "chromeos-fonts-0.0.1-r52.tbz2",
        sha256 = "40128c1465aa6ca717561be8acda996478cc41aa414fda18d64cbaa37509c02c",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/board/arm64-generic/postsubmit-R107-15066.0.0-38990-8804972997102079969/packages/chromeos-base/chromeos-fonts-0.0.1-r52.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_binutils_2_36_1_r8",
        downloaded_file_path = "binutils-2.36.1-r8.tbz2",
        sha256 = "c790efb90da825d0c169c34e191290826e03e21031c6c993e1d337b8b9c7d042",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/binutils-2.36.1-r8.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_compiler_rt_15_0_pre458507_r6",
        downloaded_file_path = "compiler-rt-15.0_pre458507-r6.tbz2",
        sha256 = "39723dbd256b02ec19b9293a18139a12d6764c083380a90bdb28e13471690727",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/compiler-rt-15.0_pre458507-r6.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_gcc_10_2_0_r28",
        downloaded_file_path = "gcc-10.2.0-r28.tbz2",
        sha256 = "bca12617716fc725143a34894b9eb45116061e6e37fbd4a3fcf28bb67660b395",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/gcc-10.2.0-r28.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_gdb_9_2_20200923_r9",
        downloaded_file_path = "gdb-9.2.20200923-r9.tbz2",
        sha256 = "ec4d7d2a0bf54872fc7de56777f6d80773a88dada52eb88cb3b2eccf88ac9dac",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/gdb-9.2.20200923-r9.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_glibc_2_33_r17",
        downloaded_file_path = "glibc-2.33-r17.tbz2",
        sha256 = "8a21f6c510bdbead7d86351a24cd5362272cb53044be13e0a1d482c66c24f1ae",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/glibc-2.33-r17.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_go_1_18_r2",
        downloaded_file_path = "go-1.18-r2.tbz2",
        sha256 = "3b3d0066a46a7cc535eaf60a1c23aeaca7b4ff3b6edd565c6f2f31ef1b470ba0",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/go-1.18-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_libcxx_15_0_pre458507_r6",
        downloaded_file_path = "libcxx-15.0_pre458507-r6.tbz2",
        sha256 = "795a65043849b065741a8cc50119b5f331d3406cb84263b975a370186e0b7344",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/libcxx-15.0_pre458507-r6.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_libxcrypt_4_4_28_r1",
        downloaded_file_path = "libxcrypt-4.4.28-r1.tbz2",
        sha256 = "a925eed789030a8431084abb8b6e1c985ec0179e26fd83f266c4fa183f135b41",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/libxcrypt-4.4.28-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_linux_headers_4_14_r56",
        downloaded_file_path = "linux-headers-4.14-r56.tbz2",
        sha256 = "b79b881f88ff8c639dfa0a012aeda61f6041c169d946fc5071298b8cd23ea597",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/linux-headers-4.14-r56.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_aarch64_cros_linux_gnu_llvm_libunwind_15_0_pre458507_r4",
        downloaded_file_path = "llvm-libunwind-15.0_pre458507-r4.tbz2",
        sha256 = "af0295d8ce5d8c3621864c8a328193d493afa3984ccebe8fd6afa3f46bc6b855",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-aarch64-cros-linux-gnu/llvm-libunwind-15.0_pre458507-r4.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_binutils_2_36_1_r8",
        downloaded_file_path = "binutils-2.36.1-r8.tbz2",
        sha256 = "d26d09eee499a42b3c7889b34621b8c6021e8f76da75df80d8cabbd9f2eeeba8",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-armv7a-cros-linux-gnueabihf/binutils-2.36.1-r8.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_compiler_rt_15_0_pre458507_r6",
        downloaded_file_path = "compiler-rt-15.0_pre458507-r6.tbz2",
        sha256 = "33dd045111be165c3af39c035c07c825145604c6e6dde39be640b255d4418611",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-armv7a-cros-linux-gnueabihf/compiler-rt-15.0_pre458507-r6.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_gcc_10_2_0_r28",
        downloaded_file_path = "gcc-10.2.0-r28.tbz2",
        sha256 = "23d8b96c583c8284d45146dba7cd31a8271ffbc6991713f289b2ddaddf6efdac",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-armv7a-cros-linux-gnueabihf/gcc-10.2.0-r28.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_gdb_9_2_20200923_r9",
        downloaded_file_path = "gdb-9.2.20200923-r9.tbz2",
        sha256 = "78131596711c47141e9e64e961bada04268c2679b4f283df3441b9a1762e6440",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-armv7a-cros-linux-gnueabihf/gdb-9.2.20200923-r9.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_glibc_2_33_r17",
        downloaded_file_path = "glibc-2.33-r17.tbz2",
        sha256 = "ae3893b911af0ae6e5a7afdf187866b7e5b5f01a7c6cf25a780fa89c3177fb3f",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-armv7a-cros-linux-gnueabihf/glibc-2.33-r17.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_go_1_18_r2",
        downloaded_file_path = "go-1.18-r2.tbz2",
        sha256 = "146d524dbfae84da223693aefec3b86bfbdd7a4f9ab6be7d3b41feeb663e46fa",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-armv7a-cros-linux-gnueabihf/go-1.18-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_libcxx_15_0_pre458507_r6",
        downloaded_file_path = "libcxx-15.0_pre458507-r6.tbz2",
        sha256 = "4b82d1057d031559ded601c92f25149d834c2773f90b8e6e227b656d240e6fcb",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-armv7a-cros-linux-gnueabihf/libcxx-15.0_pre458507-r6.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_libxcrypt_4_4_28_r1",
        downloaded_file_path = "libxcrypt-4.4.28-r1.tbz2",
        sha256 = "f2ca1bc9f21b7800fc27f392d299f13b3744afca97f5a89d86e1b186828072c3",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-armv7a-cros-linux-gnueabihf/libxcrypt-4.4.28-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_linux_headers_4_14_r56",
        downloaded_file_path = "linux-headers-4.14-r56.tbz2",
        sha256 = "3c7a775af44c9dc6ec086ecb11e180437c6299527d412e162ba80e729cb9f7ea",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-armv7a-cros-linux-gnueabihf/linux-headers-4.14-r56.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_armv7a_cros_linux_gnueabihf_llvm_libunwind_15_0_pre458507_r4",
        downloaded_file_path = "llvm-libunwind-15.0_pre458507-r4.tbz2",
        sha256 = "02452104a0260b46dcd17f3432d026bcf00d083a99e193dca6c7d06f1eacac00",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-armv7a-cros-linux-gnueabihf/llvm-libunwind-15.0_pre458507-r4.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_binutils_2_36_1_r8",
        downloaded_file_path = "binutils-2.36.1-r8.tbz2",
        sha256 = "02bebcd7e6a914f4dcc01c945e54ab0cb3d63542ebef1db95effa588fdff14e0",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-x86_64-cros-linux-gnu/binutils-2.36.1-r8.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_gcc_10_2_0_r28",
        downloaded_file_path = "gcc-10.2.0-r28.tbz2",
        sha256 = "56f8c191189b2c275264cc835bc8d9750c8cf955bbf58f6904bd44d31a1c8a37",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-x86_64-cros-linux-gnu/gcc-10.2.0-r28.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_gdb_9_2_20200923_r9",
        downloaded_file_path = "gdb-9.2.20200923-r9.tbz2",
        sha256 = "c27110fb16022d5de1a8a1fd5ecdf41bcb904ecd92f82382acbc82ba6a18abb8",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-x86_64-cros-linux-gnu/gdb-9.2.20200923-r9.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_glibc_2_33_r17",
        downloaded_file_path = "glibc-2.33-r17.tbz2",
        sha256 = "2ccdc93c03014852313fab0338eb47718d16b1fafa5b9ada167d7fdb54bdd835",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-x86_64-cros-linux-gnu/glibc-2.33-r17.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_go_1_18_r2",
        downloaded_file_path = "go-1.18-r2.tbz2",
        sha256 = "51dc9a0ca9aec5d83bfb7c78e076ed1c600408bd89e6af26974dc009ce1f32ac",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-x86_64-cros-linux-gnu/go-1.18-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_libcxx_15_0_pre458507_r6",
        downloaded_file_path = "libcxx-15.0_pre458507-r6.tbz2",
        sha256 = "dbb7be9902e1d5e9c3d9ea949ef35063e87046dd69ae16dd548da4bbbf84a26e",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-x86_64-cros-linux-gnu/libcxx-15.0_pre458507-r6.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_libxcrypt_4_4_28_r1",
        downloaded_file_path = "libxcrypt-4.4.28-r1.tbz2",
        sha256 = "1d79c4a8bb4029b7f16909f24a600cd37492f731a636a325f869ddf69d20f171",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-x86_64-cros-linux-gnu/libxcrypt-4.4.28-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_linux_headers_4_14_r56",
        downloaded_file_path = "linux-headers-4.14-r56.tbz2",
        sha256 = "45ecd4689b2f31c0dad26c9d1bac7515e7548ea5ffaf6252dcf6f8eeb9dd2d8b",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-x86_64-cros-linux-gnu/linux-headers-4.14-r56.tbz2"],
    )
    http_file(
        name = "amd64_host_cross_x86_64_cros_linux_gnu_llvm_libunwind_15_0_pre458507_r4",
        downloaded_file_path = "llvm-libunwind-15.0_pre458507-r4.tbz2",
        sha256 = "903138588072c3c6dac347082e3aa9a6dc7e65224a46b85a81ed06c3dcf924d2",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/cross-x86_64-cros-linux-gnu/llvm-libunwind-15.0_pre458507-r4.tbz2"],
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
        name = "amd64_host_coreboot_sdk_0_0_1_r116",
        downloaded_file_path = "coreboot-sdk-0.0.1-r116.tbz2",
        sha256 = "a9f4e06058918a201a5aa5a9ab55be2ae9695e4ec47a731633fc92442a10cdaf",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/dev-embedded/coreboot-sdk-0.0.1-r116.tbz2"],
    )
    http_file(
        name = "amd64_host_xcb_proto_1_14_1",
        downloaded_file_path = "xcb-proto-1.14.1.tbz2",
        sha256 = "848f74ec91f249c11ca462729ede8136190e8fbdce647782c8d0b2fd2531a2f9",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/x11-base/xcb-proto-1.14.1.tbz2"],
    )

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
