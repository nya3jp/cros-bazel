# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

visibility("//ebuild/private/...")

def _primordial_transition_impl(settings, attr):
    _ignore = (settings, attr)
    return {
        "//bazel/module_extensions/toolchains:primordial": True,
        "//command_line_option:platforms": [Label("//bazel/platforms:host")],
    }

# Ideally this should be placed as an input transition on every target that
# needs to be built before the bootstrapped toolchain has finished building.
# If we were to miss a target, then it will still work, but then the targets we
# missed would likely be built twice.
primordial_transition = transition(
    implementation = _primordial_transition_impl,
    inputs = [],
    outputs = [
        "//bazel/module_extensions/toolchains:primordial",
        "//command_line_option:platforms",
    ],
)
