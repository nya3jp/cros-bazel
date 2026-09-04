# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/portage/build_defs:common.bzl", "BinaryPackageInfo", "SysrootInfo")

def _sysroot_create_impl(ctx):
    log = ctx.actions.declare_file(ctx.label.name + ".log")
    output = ctx.actions.declare_file(ctx.label.name + ".output")

    args = ctx.actions.args()
    args.add_all([
        "--log",
        log,
        "--temp-dir",
        log.dirname + "/tmp",
        ctx.executable._setup_board,
        "-b",
        ctx.attr.board,
        "-o",
        output,
    ])

    inputs = [
        ctx.executable._setup_board,
        # Forces the action to always run.
        ctx.file._cache_bust,
    ]

    for pkg in ctx.attr.toolchain_pkgs:
        info = pkg[BinaryPackageInfo]
        inputs.append(info.partial)
        args.add("--toolchain-pkg", "%s:%s:%s" % (info.category, info.package_name, info.partial.path))

    ctx.actions.run(
        executable = ctx.executable._action_wrapper,
        inputs = inputs,
        arguments = [args],
        outputs = [output, log],
        execution_requirements = {
            # The action needs to run against the permanent SDK, so must
            # be run locally.
            # This implies no-sandbox and no-remote.
            "local": "1",
            "no-cache": "1",
        },
        mnemonic = "SysrootCreate",
        progress_message = "Creating sysroot /build/%s" % ctx.attr.board,
    )

    return [
        OutputGroupInfo(logs = [log]),
        SysrootInfo(output = output),
    ]

sysroot_create = rule(
    implementation = _sysroot_create_impl,
    doc = "Replace the sysroot in the permanent SDK with a fresh one.",
    attrs = dict(
        board = attr.string(
            mandatory = True,
            doc = """
            The target board name.
            """,
        ),
        toolchain_pkgs = attr.label_list(
            providers = [BinaryPackageInfo],
            doc = """
            Host cross toolchain binary packages (e.g. cross-${CHOST}/glibc)
            to stage into the sysroot cache.
            """,
        ),
        _setup_board = attr.label(
            default = "//bazel/portage/build_defs:setup_board",
            executable = True,
            cfg = "exec",
        ),
        _action_wrapper = attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/portage/bin/action_wrapper"),
        ),
        _cache_bust = attr.label(
            default = Label("//bazel:now"),
            allow_single_file = True,
        ),
    ),
    provides = [SysrootInfo],
)
