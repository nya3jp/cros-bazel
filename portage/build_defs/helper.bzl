# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
Provides generic helpers for BUILD.bazel.
"""

load("@portage//:settings.bzl", "BOARD")
load("//bazel/build_defs:always_fail.bzl", "always_fail")

visibility("public")

def if_board_is_set(rule, *, name, **kwargs):
    """
    Conditionally calls a rule function when a target board is set.

    If a target board is unset, defines a fake target that always fails.
    """
    if BOARD == None:
        always_fail(name = name, message = "$BOARD is not set")
    else:
        rule(name = name, **kwargs)
