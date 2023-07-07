# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@bazel_skylib//rules:common_settings.bzl", _bool_flag = "bool_flag")
load("@bazel_skylib//lib:selects.bzl", "selects")

visibility("public")

def config_setting_not(name, target, visibility = None):
    native.alias(
        name = name,
        actual = select({
            target: "//bazel/build_defs/bool_flag:always_false",
            "//conditions:default": "//bazel/build_defs/bool_flag:always_true",
        }),
        visibility = visibility,
    )

def config_setting_all(name, targets, visibility = None):
    selects.config_setting_group(
        name = name,
        match_all = targets,
        visibility = visibility,
    )

def config_setting_any(name, targets, visibility = None):
    selects.config_setting_group(
        name = name,
        match_any = targets,
        visibility = visibility,
    )

def config_setting_not_all(name, targets, visibility = None):
    all_name = "_{name}_all".format(name = name)
    config_setting_all(
        name = all_name,
        targets = targets,
        visibility = ["//visibility:private"],
    )
    config_setting_not(
        name = name,
        target = all_name,
        visibility = visibility,
    )

def config_setting_mutually_exclusive(name, target, mutually_exclusive_with, visibility = None):
    nand_name = "_{name}_not_all".format(name = name)
    config_setting_not_all(
        name = nand_name,
        targets = [target, mutually_exclusive_with],
        visibility = ["//visibility:private"],
    )
    native.alias(
        name = name,
        actual = select(
            {nand_name: target},
            no_match_error = "{lhs} and {rhs} are mutually exclusive".format(
                lhs = target,
                rhs = mutually_exclusive_with,
            ),
        ),
        visibility = visibility,
    )

def config_setting_requires(name, target, requires, error = None, visibility = None):
    # Valid iff (not target) OR requires
    inverted = "_{name}_not_target".format(name = name)
    config_setting_not(
        name = inverted,
        target = target,
        visibility = ["//visibility:private"],
    )

    valid = "_{name}_valid".format(name = name)
    config_setting_any(
        name = valid,
        targets = [inverted, requires],
        visibility = ["//visibility:private"],
    )

    native.alias(
        name = name,
        actual = select(
            {valid: target},
            no_match_error = error or "{target} requires {requires}".format(
                target = target,
                requires = requires,
            ),
        ),
        visibility = visibility,
    )

def bool_flag(name, default, custom_condition = False, visibility = None):
    """Creates a bool flag and a {name}_enabled/_disabled config_setting.
    Args:
        name: (str) The name of your flag
        default: (bool) Whether the flag is enabled by default
        custom_condition: (bool) If true, generates _{name}_enabled instead of
          {name}_enabled, and you're required to define your own {name}_enabled.
        visibility: The visibility of the config_settings.
    """
    _bool_flag(
        name = name,
        build_setting_default = default,
        visibility = ["//visibility:private"],
    )

    native.config_setting(
        name = "{prefix}{name}_enabled".format(
            prefix = "_" if custom_condition else "",
            name = name,
        ),
        flag_values = {
            ":{name}".format(name = name): "True",
        },
        visibility = ["//visibility:private"] if custom_condition else visibility,
    )

    config_setting_not(
        name = "{name}_disabled".format(name = name),
        target = ":{name}_enabled".format(name = name),
        visibility = visibility,
    )
