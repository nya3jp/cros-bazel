# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("@rules_perl//perl:deps.bzl", "perl_register_toolchains", "perl_rules_dependencies")

# Must be seperated from language_repositories, as the loads above will fail if they haven't been downloaded yet.

def load_toolchains():
    perl_rules_dependencies()
    perl_register_toolchains()
