# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Rules for creating hermetic C++ binaries"""

load(
    "//bazel/module_extensions/toolchains/hermetic_launcher:hermetic_launcher.bzl",
    "create_hermetic_launcher_nontest",
    "create_hermetic_launcher_test",
)

visibility("public")

# https://bazel.build/reference/be/common-definitions#common-attributes
_COMMON_BUILD_ARGS = [
    "compatible_with",
    "deprecation",
    "distribs",
    "exec_compatible_with",
    "exec_properties",
    "features",
    "restricted_to",
    "tags",
    "target_compatible_with",
    "testonly",
    "toolchains",
    "visibility",
]

# https://bazel.build/reference/be/common-definitions#common-attributes-binaries
_COMMON_BIN_ARGS = [
    "args",
    "env",
    "output_licenses",
]

# https://bazel.build/reference/be/common-definitions#common-attributes-tests
_COMMON_TEST_ARGS = [
    "args",
    "env",
    "env_inherit",
    "size",
    "timeout",
    "flaky",
    "shard_count",
    "local",
]

def _hermetic_launcher(wrapper_rule):
    """Generates a macro that wraps a rule to ensure it runs hermetically.

    Args:
        wrapper_rule: rule: The rule to wrap (eg. cc_binary).
    Returns:
        A macro wrapping the rule with a hermetic launcher.
    """

    def wrapper(name, visibility = None, features = [], hermetic_launcher = True, **kwargs):
        # Shared libraries don't need a launcher.
        if kwargs.get("linkshared", False) or not hermetic_launcher:
            # buildifier: disable=native-cc
            native.cc_binary(
                name = name,
                visibility = visibility,
                features = features,
                **kwargs
            )
        else:
            # The hard part here is determining which kwargs should go to the
            # cc_binary rule, which should go to the launcher, and which should go
            # to both.
            wrapper_args = {}
            inner_args = {}
            for k, v in kwargs.items():
                if k in _COMMON_BUILD_ARGS or k in _COMMON_BIN_ARGS:
                    # Attributes such as testonly are relevant for both the inner
                    # and outer rules.
                    wrapper_args[k] = v
                    inner_args[k] = v
                elif k in _COMMON_TEST_ARGS:
                    # If this is a non-test rule, this allows bazel itself to handle
                    # the error.
                    wrapper_args[k] = v
                else:
                    inner_args[k] = v

            real_name = "_%s_real" % name

            # buildifier: disable=native-cc
            native.cc_binary(
                name = real_name,
                visibility = ["//visibility:private"],
                features = features,
                **inner_args
            )

            wrapper_rule(
                name = name,
                bin = real_name,
                enable = "@@//bazel/module_extensions/toolchains/cc:use_hermetic_launcher",
                visibility = visibility,
                **wrapper_args
            )

    return wrapper

cc_binary = _hermetic_launcher(create_hermetic_launcher_nontest)
cc_test = _hermetic_launcher(create_hermetic_launcher_test)
