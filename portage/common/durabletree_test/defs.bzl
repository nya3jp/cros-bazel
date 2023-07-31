# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/bash:defs.bzl", "BASH_RUNFILES_ATTR", "wrap_binary_with_args")

def _durabletree_test_impl(ctx):
    output_dir = ctx.actions.declare_directory(ctx.attr.name + ".durabletree")
    output_script = ctx.actions.declare_file(ctx.attr.name + ".sh")

    execution_requirements = {}
    if not ctx.attr.sandbox_on_generate:
        execution_requirements["no-sandbox"] = ""

    ctx.actions.run(
        inputs = depset(),
        outputs = [output_dir],
        executable = ctx.executable.bin,
        arguments = ["generate", output_dir.path],
        execution_requirements = execution_requirements,
    )

    return wrap_binary_with_args(
        ctx,
        out = output_script,
        binary = ctx.attr.bin,
        args = ["check", output_dir.short_path],
        content_prefix = "export RUST_BACKTRACE=1",
        runfiles = ctx.runfiles(files = [ctx.executable.bin, output_dir]),
    )

durabletree_test = rule(
    implementation = _durabletree_test_impl,
    test = True,
    attrs = {
        "bin": attr.label(
            mandatory = True,
            executable = True,
            cfg = "exec",
        ),
        "sandbox_on_generate": attr.bool(
            mandatory = True,
        ),
        "_bash_runfiles": BASH_RUNFILES_ATTR,
    },
)
