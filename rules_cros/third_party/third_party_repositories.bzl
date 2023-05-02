# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//rules_cros/third_party/e2fsprogs:e2fsprogs_repositories.bzl", "e2fsprogs_repositories")
load("//rules_cros/third_party/fuse:fuse_repositories.bzl", "fuse_repositories")
load("//rules_cros/third_party/lz4:lz4_repositories.bzl", "lz4_repositories")
load("//rules_cros/third_party/openssl:openssl_repositories.bzl", "openssl_repositories")
load("//rules_cros/third_party/squashfuse:squashfuse_repositories.bzl", "squashfuse_repositories")

def third_party_repositories():
    e2fsprogs_repositories()
    fuse_repositories()
    lz4_repositories()
    openssl_repositories()
    squashfuse_repositories()
