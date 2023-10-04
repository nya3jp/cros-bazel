# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

visibility("//bazel/module_extensions")

def portage_testdata_prebuilts(prebuilt_binpkg):
    prebuilt_binpkg(
        name = "testdata_ncurses",
        url = "gs://chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.12.050023/packages/sys-libs/ncurses-6.3_p20220423-r1.tbz2",
        runtime_deps = ["@files//:testdata_glibc_alias"],
        slot = "0/6",
    )

    prebuilt_binpkg(
        name = "testdata_glibc",
        url = "gs://chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.12.050023/packages/sys-libs/glibc-2.35-r22.tbz2",
        runtime_deps = [],
        slot = "2.2/2.2",
    )

    prebuilt_binpkg(
        name = "testdata_nano",
        url = "gs://chromeos-prebuilt/host/amd64/amd64-host/chroot-2023.09.12.050023/packages/app-editors/nano-6.4.tbz2",
        runtime_deps = [
            "@files//:testdata_glibc_alias",
            "@files//:testdata_ncurses_alias",
        ],
        slot = "0/0",
    )
