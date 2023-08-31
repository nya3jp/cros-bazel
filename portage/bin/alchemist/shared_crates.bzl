# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

visibility("private")

SHARED_CRATES = [
    "//bazel/portage/common/chrome-trace:srcs",
    "//bazel/portage/common/cliutil:srcs",
    "//bazel/portage/common/fileutil:srcs",
    "//bazel/portage/common/portage/version:srcs",
    "//bazel/portage/common/testutil:srcs",
    "//bazel/portage/common/tracing-chrome-trace:srcs",
]
