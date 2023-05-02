# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""A module defining the third party dependency OpenSSL"""

load("//rules_cros/third_party:github_archive.bzl", "github_archive")

VERSION = "1.1.1o"
CHECKSUM = "0f745b85519aab2ce444a3dcada93311ba926aea2899596d01e7f948dbd99981"

VERSION_UNDERSCORED = VERSION.replace(".", "_")

def openssl_repositories():
    github_archive(
        name = "openssl",
        github_user = "openssl",
        github_repo = "openssl",
        build_file = Label("//rules_cros/third_party/openssl:BUILD.openssl.bazel"),
        checksum = CHECKSUM,
        tag = "OpenSSL_%s" % VERSION_UNDERSCORED,
    )
