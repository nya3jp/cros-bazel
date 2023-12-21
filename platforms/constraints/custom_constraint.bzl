# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

def custom_constraint(name, options, default):
    if default not in options:
        fail("{name}: Default ({default}) must be an option in {options}".format(
            name = name,
            default = default,
            options = options,
        ))

    native.constraint_setting(
        name = name,
        default_constraint_value = ":{}_{}".format(name, default),
    )

    for option in options:
        native.constraint_value(
            name = "{}_{}".format(name, option),
            constraint_setting = ":{}".format(name),
        )
