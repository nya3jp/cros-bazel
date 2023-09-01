# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")

def alpine_repository():
    http_file(
        name = "alpine-minirootfs",
        downloaded_file_path = "alpine-minirootfs-3.18.3-x86_64.tar.gz",
        integrity = "sha256-/FdzJLfpQ5hjEYx+UgnSXX7d6mumK1i628M8loYbnE4=",
        urls = [
            "https://dl-cdn.alpinelinux.org/alpine/v3.18/releases/x86_64/alpine-minirootfs-3.18.3-x86_64.tar.gz",
        ],
    )
