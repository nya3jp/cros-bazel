# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

package(default_visibility = ["//visibility:public"])

constraint_setting(
    name = "vendor",
    default_constraint_value = ":pc",
)

constraint_value(
    name = "cros",
    constraint_setting = ":vendor",
)

constraint_value(
    name = "pc",
    constraint_setting = ":vendor",
)
