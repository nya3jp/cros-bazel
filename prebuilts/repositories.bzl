# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")

def prebuilts_dependencies():
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
        name = "amd64_host_xcb_proto_1_14_1",
        downloaded_file_path = "xcb-proto-1.14.1.tbz2",
        sha256 = "848f74ec91f249c11ca462729ede8136190e8fbdce647782c8d0b2fd2531a2f9",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2022.08.22.085953/packages/x11-base/xcb-proto-1.14.1.tbz2"],
    )

    # ~/cros-bazel/src/bazel/tools/sdk_repos.sh  2023.03.04.201551
    http_file(
        name = "amd64_host_2023_03_04_201551_app_text_docbook_xml_dtd_4_1_2_r7",
        downloaded_file_path = "docbook-xml-dtd-4.1.2-r7.tbz2",
        sha256 = "ef12f4770b41de26e0685c4da412f12cc001ec1a3e5067d2b67c7edf7ab81039",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/app-text/docbook-xml-dtd-4.1.2-r7.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_app_text_docbook_xml_dtd_4_2_r3",
        downloaded_file_path = "docbook-xml-dtd-4.2-r3.tbz2",
        sha256 = "62d1e99133e2f8b41c1da1ee42feb6c931eb40645cfaa54192ca2e56d3da14a8",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/app-text/docbook-xml-dtd-4.2-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_app_text_docbook_xml_dtd_4_3_r2",
        downloaded_file_path = "docbook-xml-dtd-4.3-r2.tbz2",
        sha256 = "9828fd66ca1ed3d278d077651e0f4e5e07c9605829cd2de5a1b1c267b11e5b8e",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/app-text/docbook-xml-dtd-4.3-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_app_text_docbook_xml_dtd_4_4_r3",
        downloaded_file_path = "docbook-xml-dtd-4.4-r3.tbz2",
        sha256 = "9ccb0765874ded4c043df0a10cd64b659b71fd30d2a96b3ebb54b5aabd2de721",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/app-text/docbook-xml-dtd-4.4-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_app_text_docbook_xml_dtd_4_5_r2",
        downloaded_file_path = "docbook-xml-dtd-4.5-r2.tbz2",
        sha256 = "e2d8ac650e6d7012046c29c95e36646f3442c40d558dfbb85a600a1b1a40aff2",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/app-text/docbook-xml-dtd-4.5-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_cross_aarch64_cros_linux_gnu_binutils_2_36_1_r9",
        downloaded_file_path = "binutils-2.36.1-r9.tbz2",
        sha256 = "58675375b7d2070fb84039c5eaca005a5f6474e49975d37e7bf31b7c1f04e518",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/cross-aarch64-cros-linux-gnu/binutils-2.36.1-r9.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_cross_aarch64_cros_linux_gnu_compiler_rt_16_0_pre475826_r3",
        downloaded_file_path = "compiler-rt-16.0_pre475826-r3.tbz2",
        sha256 = "c36c9ebcfef31a6b2ce1d94c4f429414c4cc7933f28dcc25ca9b3bacf9c6ce6b",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/cross-aarch64-cros-linux-gnu/compiler-rt-16.0_pre475826-r3.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_cross_aarch64_cros_linux_gnu_gcc_10_2_0_r30",
        downloaded_file_path = "gcc-10.2.0-r30.tbz2",
        sha256 = "1f702c681782e0c86a479723c8e2536025c3b6cdb1b4283a62e3b0c74e17dcd9",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/cross-aarch64-cros-linux-gnu/gcc-10.2.0-r30.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_cross_aarch64_cros_linux_gnu_gdb_11_2_r2",
        downloaded_file_path = "gdb-11.2-r2.tbz2",
        sha256 = "4cac8a7dc77289d30c28b8fb667d3141e526af1e9672180d9be480aa7bab6d69",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/cross-aarch64-cros-linux-gnu/gdb-11.2-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_cross_aarch64_cros_linux_gnu_glibc_2_35_r17",
        downloaded_file_path = "glibc-2.35-r17.tbz2",
        sha256 = "f44eabd8bd5295331e3d4664673807ec64b784c09d2ee2179378466e92feebf2",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/cross-aarch64-cros-linux-gnu/glibc-2.35-r17.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_cross_aarch64_cros_linux_gnu_go_1_19_4_r1",
        downloaded_file_path = "go-1.19.4-r1.tbz2",
        sha256 = "a08867d88fc288476f25c6c61d45716aa8e5b28a284b46584b39f2fd4eb16180",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/cross-aarch64-cros-linux-gnu/go-1.19.4-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_cross_aarch64_cros_linux_gnu_libcxx_16_0_pre475826_r2",
        downloaded_file_path = "libcxx-16.0_pre475826-r2.tbz2",
        sha256 = "cdd9122a4d5c7fabc911eea3c2e952527151abeb019b39ed91d49f0adc3fd689",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/cross-aarch64-cros-linux-gnu/libcxx-16.0_pre475826-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_cross_aarch64_cros_linux_gnu_libxcrypt_4_4_28_r2",
        downloaded_file_path = "libxcrypt-4.4.28-r2.tbz2",
        sha256 = "f87785534087344bc3243036670195eb0504894fafcaa0059a5a440c123e612f",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/cross-aarch64-cros-linux-gnu/libxcrypt-4.4.28-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_cross_aarch64_cros_linux_gnu_linux_headers_4_14_r61",
        downloaded_file_path = "linux-headers-4.14-r61.tbz2",
        sha256 = "1e88fd59feb492d1ce0093924a628b641bcd4832bbf7ea6260c154b9997a7e19",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/cross-aarch64-cros-linux-gnu/linux-headers-4.14-r61.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_cross_aarch64_cros_linux_gnu_llvm_libunwind_16_0_pre475826_r2",
        downloaded_file_path = "llvm-libunwind-16.0_pre475826-r2.tbz2",
        sha256 = "e40d912ac7a689c8e4d9aa0eb92e901984a8bc8215ed730c984a278a7544bdb4",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/cross-aarch64-cros-linux-gnu/llvm-libunwind-16.0_pre475826-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_cross_x86_64_cros_linux_gnu_binutils_2_36_1_r9",
        downloaded_file_path = "binutils-2.36.1-r9.tbz2",
        sha256 = "135e2997bed5194dc2249911b9ed8a4059fcf59465c4b7bcb157501f0b69efd9",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/cross-x86_64-cros-linux-gnu/binutils-2.36.1-r9.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_cross_x86_64_cros_linux_gnu_gcc_10_2_0_r30",
        downloaded_file_path = "gcc-10.2.0-r30.tbz2",
        sha256 = "bf940deff1a9ec98c97b4f6cd1370c40c43625ce7fd166d670be314ab8334220",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/cross-x86_64-cros-linux-gnu/gcc-10.2.0-r30.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_cross_x86_64_cros_linux_gnu_gdb_11_2_r2",
        downloaded_file_path = "gdb-11.2-r2.tbz2",
        sha256 = "3c8d2e555a58f881715137c1b42dc91208977ff0c85fd2a19e9f172604f0a3bd",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/cross-x86_64-cros-linux-gnu/gdb-11.2-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_cross_x86_64_cros_linux_gnu_glibc_2_35_r17",
        downloaded_file_path = "glibc-2.35-r17.tbz2",
        sha256 = "3b3fe58c4049ee814833132e2a582bd80c958e15774320fd87d184b3ee839ec5",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/cross-x86_64-cros-linux-gnu/glibc-2.35-r17.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_cross_x86_64_cros_linux_gnu_go_1_19_4_r1",
        downloaded_file_path = "go-1.19.4-r1.tbz2",
        sha256 = "3f987165dbb4bcf82db98dc59c85dc38b960a9c83add06a6c8a776e7f3339200",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/cross-x86_64-cros-linux-gnu/go-1.19.4-r1.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_cross_x86_64_cros_linux_gnu_libcxx_16_0_pre475826_r2",
        downloaded_file_path = "libcxx-16.0_pre475826-r2.tbz2",
        sha256 = "afee3115b2a5b66590e397b53ddfd342c5267e6633889e5046f89235e72a9568",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/cross-x86_64-cros-linux-gnu/libcxx-16.0_pre475826-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_cross_x86_64_cros_linux_gnu_libxcrypt_4_4_28_r2",
        downloaded_file_path = "libxcrypt-4.4.28-r2.tbz2",
        sha256 = "0990042751cbe5a43a4fcf29b413386d31903d60576f6dede4e07d42a652f710",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/cross-x86_64-cros-linux-gnu/libxcrypt-4.4.28-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_cross_x86_64_cros_linux_gnu_linux_headers_4_14_r61",
        downloaded_file_path = "linux-headers-4.14-r61.tbz2",
        sha256 = "82f587677ecf5fe26c379bfef09adb85e667a3ea6f7f51fb8c28b3869b5e7d1a",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/cross-x86_64-cros-linux-gnu/linux-headers-4.14-r61.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_cross_x86_64_cros_linux_gnu_llvm_libunwind_16_0_pre475826_r2",
        downloaded_file_path = "llvm-libunwind-16.0_pre475826-r2.tbz2",
        sha256 = "d5d73aa15d49c22d2611367f50475a4363016bce4f1e0c956bd87d058387483f",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/cross-x86_64-cros-linux-gnu/llvm-libunwind-16.0_pre475826-r2.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_dev_embedded_hps_sdk_0_0_1_r5",
        downloaded_file_path = "hps-sdk-0.0.1-r5.tbz2",
        sha256 = "37c1f6863b117dfc7b5f60f12f2bd83670beb834a5edcf4d1b091ebff18d4831",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/dev-embedded/hps-sdk-0.0.1-r5.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_dev_lang_rust_1_67_1",
        downloaded_file_path = "rust-1.67.1.tbz2",
        sha256 = "39708af90f5dfa423d8e455079a2ceda1bbac0fa021093f7b39e5659450d5165",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/dev-lang/rust-1.67.1.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_dev_lang_rust_bootstrap_1_66_0",
        downloaded_file_path = "rust-bootstrap-1.66.0.tbz2",
        sha256 = "bbd243f26705f266743d788bd8fc289ff10cbaf197fbb4d9220772edf53bb66b",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/dev-lang/rust-bootstrap-1.66.0.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_dev_lang_rust_host_1_67_1",
        downloaded_file_path = "rust-host-1.67.1.tbz2",
        sha256 = "3ea06676aced236787501b817bc7b82024724eac10d0a7882be0c99e728a5272",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/dev-lang/rust-host-1.67.1.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_dev_lang_rust_llvm_sources_1_67_1",
        downloaded_file_path = "rust-llvm-sources-1.67.1.tbz2",
        sha256 = "56fe8631584d6b8ece1ec1a64927e3c847deb392c27aeb213c407a6c8e6f33d1",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/dev-lang/rust-llvm-sources-1.67.1.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_dev_util_b2_4_9_3",
        downloaded_file_path = "b2-4.9.3.tbz2",
        sha256 = "9ee3363e98b228727eeff5c678743299a02ac2099053e49955fd7589ac6df988",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/dev-util/b2-4.9.3.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_dev_util_meson_format_array_0",
        downloaded_file_path = "meson-format-array-0.tbz2",
        sha256 = "a51bf2e76402c5526224d09d3b501b1c86f569d8274f6d8fe730edfd82602f37",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/dev-util/meson-format-array-0.tbz2"],
    )
    http_file(
        name = "amd64_host_2023_03_04_201551_x11_base_xcb_proto_1_15_2",
        downloaded_file_path = "xcb-proto-1.15.2.tbz2",
        sha256 = "e106006c7dcd0c1a69c88d2d4c7a2abd27542b7b46f59a25d207751180eeb619",
        urls = ["https://commondatastorage.googleapis.com/chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.03.04.201551/packages/x11-base/xcb-proto-1.15.2.tbz2"],
    )
