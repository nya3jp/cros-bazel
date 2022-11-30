# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")
load("//bazel/third_party:github_archive.bzl", "github_archive")

RULES_GO_VERSION = "0.36.0"
RULES_GO_CHECKSUM = "667aa901ff13d19b2ed56534f8d4a99903534e7a65be24264c7f182cf0af3d7b"

GAZELLE_VERSION = "0.28.0"
GAZELLE_CHECKSUM = "ba9636053dce44cd908b6eff44ab0801ffad7f26fad38ba20c9b8587bae435df"

def rules_ebuild_repositories():
    github_archive(
        name = "io_bazel_rules_go",
        checksum = RULES_GO_CHECKSUM,
        github_user = "bazelbuild",
        github_repo = "rules_go",
        tag = "v{version}".format(version = RULES_GO_VERSION),
        strip_prefix = "rules_go-{version}".format(version = RULES_GO_VERSION),
    )
    github_archive(
        name = "bazel_gazelle",
        checksum = GAZELLE_CHECKSUM,
        github_user = "bazelbuild",
        github_repo = "bazel-gazelle",
        tag = "v{version}".format(version = GAZELLE_VERSION),
        strip_prefix = "bazel-gazelle-{version}".format(version = GAZELLE_VERSION),
    )
