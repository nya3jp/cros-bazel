# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""Wrapper macros and rules to create a fully hermetic binary."""

load("@bazel_skylib//rules:common_settings.bzl", "BuildSettingInfo")

visibility("public")

HERMETIC_ATTRS = dict(
    _create_hermetic_launcher = attr.label(
        default = "@@//bazel/module_extensions/toolchains/hermetic_launcher:create_hermetic_launcher",
        executable = True,
        cfg = "exec",
    ),
    _hermetic_interp = attr.label(
        default = "@@//bazel/module_extensions/toolchains/files:interp",
        allow_single_file = True,
    ),
    _hermetic_libs = attr.label(
        default = "@@//bazel/module_extensions/toolchains/files:libs",
        allow_files = True,
    ),
)

def hermetic_defaultinfo(ctx, files, runfiles, executable, enabled = True, out = None):
    """Creates a DefaultInfo that creates a hermetic launcher for a given executable.

    Args:
        ctx: The rule ctx.
        files: (depset[File]) The output files.
        runfiles: (runfiles) The output runfiles.
        executable: (File) The exxecutable to wrap.
        enabled: (bool) If disabled, runs this function in compatibility mode,
          where we simply create a symlink to the original file.
        out: Optional[File] If provided, this is the file that will be output to.

    Returns:
        A DefaultInfo containing the hermetic launcher.
    """
    out = out or ctx.actions.declare_file(ctx.label.name)

    files = [out] + files.to_list()
    if executable in files:
        files.remove(executable)

    if not enabled:
        ctx.actions.symlink(output = out, target_file = executable)
        return DefaultInfo(
            files = depset(files),
            runfiles = runfiles.merge(files = [out]),
            executable = out,
        )

    args = ctx.actions.args()
    args.add_all([executable, out, ctx.file._hermetic_interp])
    args.add_all(ctx.files._hermetic_libs)
    ctx.actions.run(
        executable = ctx.executable._create_hermetic_launcher,
        inputs = [executable, ctx.file._hermetic_interp] + ctx.files._hermetic_libs,
        outputs = [out],
        arguments = [args],
    )

    runfiles_files = [f for f in runfiles.files.to_list() if f != executable]
    runfiles_files.append(out)
    runfiles = ctx.runfiles(
        files = runfiles_files,
        symlinks = runfiles.symlinks,
        root_symlinks = runfiles.root_symlinks,
    )

    return DefaultInfo(
        files = depset(files),
        runfiles = runfiles.merge(ctx.runfiles(files = [executable])),
        executable = out,
    )

def _create_hermetic_launcher_impl(ctx):
    info = ctx.attr.bin[DefaultInfo]
    return hermetic_defaultinfo(
        ctx,
        files = info.files,
        runfiles = info.default_runfiles,
        executable = ctx.executable.bin,
        enabled = ctx.attr.enable[BuildSettingInfo].value,
    )

_WRAPPER_KWARGS = dict(
    implementation = _create_hermetic_launcher_impl,
    attrs = HERMETIC_ATTRS | dict(
        bin = attr.label(mandatory = True, executable = True, cfg = "target"),
        enable = attr.label(mandatory = True, providers = [BuildSettingInfo]),
    ),
    executable = True,
)

create_hermetic_launcher_nontest = rule(test = False, **_WRAPPER_KWARGS)
create_hermetic_launcher_test = rule(test = True, **_WRAPPER_KWARGS)
