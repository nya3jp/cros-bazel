# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
load("//bazel/bash:defs.bzl", "custom_args_binary")

def regen_repo_rule_srcs(name, target, output, variable = None, extra_deps = []):
    regen_cmd = "bazel run //{pkg}:{name}".format(
        name = name,
        pkg = native.package_name(),
    )
    args = ["--target", target, "--output", output, "--regen_cmd", regen_cmd]
    if variable:
        args.extend(["--variable", variable])
    if extra_deps:
        args.append("--extra_dep")
        for dep in extra_deps:
            args.append(dep)

    custom_args_binary(
        name = name,
        binary = "//bazel/module_extensions/helpers:regen_repo_rule_srcs",
        binary_args = args,
    )
