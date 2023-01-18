# Copyright 2023 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
load(":alchemist_digest.bzl", "alchemist_digest")
load(":alchemist_repo.bzl", "alchemist_repo")

def portage_repositories():
    alchemist_digest(name = "portage_digest")
    alchemist_repo(name = "portage")
