# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//rules_cros/toolchains/nasm:repositories.bzl", "nasm_repositories")
load("//rules_cros/toolchains/perl:repositories.bzl", "perl_repositories")

def toolchain_repositories():
    nasm_repositories()
    perl_repositories()
