# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

def zstd_repository():
    http_archive(
        name = "zstd",
        build_file = "//bazel/zstd:zstd.BUILD.bazel",
        integrity = "sha256-nEOWzIKc+uMZpuJhUgLoKq1BNyBzSC/OKG+seGRtPuQ=",
        strip_prefix = "zstd-1.5.5/",
        urls = [
            "https://github.com/facebook/zstd/releases/download/v1.5.5/zstd-1.5.5.tar.gz",
        ],
    )
