# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//rules_cros/third_party:github_archive.bzl", "github_archive")

VERSION = "1.46.5"

CHECKSUM = "0286b718da1491c65c4e51453d33a25d5dad29b0964f915e627c363b4c11cb92"

def e2fsprogs_repositories():
    github_archive(
        name = "e2fsprogs",
        build_file = Label("//rules_cros/third_party:BUILD.all_srcs.bazel"),
        checksum = CHECKSUM,
        github_repo = "e2fsprogs",
        github_user = "tytso",
        strip_prefix = "e2fsprogs-%s" % VERSION,
        patches = [
            # I filed a PR, so this should hopefully be merged into upstream at
            # some point.
            # https://github.com/tytso/e2fsprogs/pull/124
            "//rules_cros/third_party/e2fsprogs:patches/support_offsets.patch",

            # fuse2fs respects permissions when chowning, so a non-sudo user
            # is unable to set an arbitrary uid / gid as owner.
            # This is precisely what we need to do in order to create a chromeos
            # image in userspace, so we patch fuse_get_context to make it think
            # we're always running as root. This should give us full control
            # over the mounted partition.
            # This should not create security issues, as the process is still
            # running in userspace.
            "//rules_cros/third_party/e2fsprogs:patches/fake_sudo.patch",
        ],
        tag = "v%s" % VERSION,
    )
