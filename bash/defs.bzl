# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

visibility("public")

_RUNFILES_HEADERS = """#!/bin/bash

# --- begin runfiles.bash initialization v3 ---
# Copy-pasted from the Bazel Bash runfiles library v3.
set -uo pipefail; set +e; f=bazel_tools/tools/bash/runfiles/runfiles.bash
source "${RUNFILES_DIR:-/dev/null}/$f" 2>/dev/null || \\
  source "$(grep -sm1 "^$f " "${RUNFILES_MANIFEST_FILE:-/dev/null}" | cut -f2- -d' ')" 2>/dev/null || \\
  source "$0.runfiles/$f" 2>/dev/null || \\
  source "$(grep -sm1 "^$f " "$0.runfiles_manifest" | cut -f2- -d' ')" 2>/dev/null || \\
  source "$(grep -sm1 "^$f " "$0.exe.runfiles_manifest" | cut -f2- -d' ')" 2>/dev/null || \\
  { echo>&2 "ERROR: cannot find $f"; exit 1; }; f=; set -e
# --- end runfiles.bash initialization v3 ---

set -e
set -uo pipefail

"""

BASH_RUNFILES_ATTRS = dict(
    _bash_runfiles = attr.label(default = "@bazel_tools//tools/bash/runfiles"),
)
BASH_RUNFILES_ATTR = attr.label(default = "@bazel_tools//tools/bash/runfiles")

def runfiles_path(ctx, file):
    """Returns a path suitable for use with the rlocation function."""
    path = file.short_path
    if path.startswith("../"):
        # The path is not to one in the root repo. Eg, it might be @portage//...
        return path.removeprefix("../")
    else:
        return "%s/%s" % (ctx.workspace_name, path)

def bash_rlocation(ctx, file):
    """Returns code that will generate the path of a file in bash."""
    return "$(rlocation '%s')" % runfiles_path(ctx, file)

def generate_bash_script(
        ctx,
        out,
        content,
        runfiles = None,
        data = []):
    ctx.actions.write(out, _RUNFILES_HEADERS + content, is_executable = True)
    runfiles = runfiles or ctx.runfiles()
    extra_runfiles = [ctx.attr._bash_runfiles[DefaultInfo].default_runfiles]
    for target in data:
        extra_runfiles.append(target[DefaultInfo].default_runfiles)
        extra_runfiles.append(ctx.runfiles(files = target[DefaultInfo].files.to_list()))
    return DefaultInfo(
        files = depset([out]),
        runfiles = runfiles.merge_all(extra_runfiles),
        executable = out,
    )

def _sh_runfiles_impl(ctx):
    # The file is likely named name + .sh already.
    out = ctx.actions.declare_file(ctx.label.name + "_generated.sh")
    return generate_bash_script(
        ctx,
        out,
        content = "source %s" % bash_rlocation(ctx, ctx.file.src),
        runfiles = ctx.runfiles(files = [ctx.file.src]),
        data = ctx.attr.data,
    )

_COMMON_ATTRS = dict(
    _bash_runfiles = attr.label(default = "@bazel_tools//tools/bash/runfiles"),
    data = attr.label_list(allow_files = True),
)

_SH_WITH_RUNFILES_ATTRS = dict(
    doc = """Same as sh_binary/test, but it imports runfiles for you so you can directly call rlocation.""",
    implementation = _sh_runfiles_impl,
    attrs = dict(
        src = attr.label(mandatory = True, allow_single_file = True),
        **_COMMON_ATTRS
    ),
)
sh_runfiles_binary = rule(executable = True, **_SH_WITH_RUNFILES_ATTRS)
sh_runfiles_test = rule(test = True, **_SH_WITH_RUNFILES_ATTRS)

_WRITE_TO_FILE = """#!/bin/bash -e

dst="$1"
shift

echo "$@" > ${dst}
"""

def wrap_binary_with_args(ctx, out, binary, args, content_prefix = "", runfiles = None, data = [], executable = None):
    """Generates a binary that runs another binary with some arguments.

    Args:
      out: (File) The executable to generate.
      binary: (Target or File) The executable to wrap.
      args: (List[str] or Args) The arguments to run it with.
      content_prefix: (Optional[str]) Any code that should run before exec'ing.
      runfiles: (Optional[runfiles]) Any files required to run your binary.
      data: List[Target] Any deps you depend on.
      executable: (Optional[File]) If provided, in the event that binary target
        contains multiple binaries, this is used to disambiguate the entrypoint.

    Returns:
      A DefaultInfo that should be able to run the binary.
    """
    if type(binary) == "Target":
        binary_files = binary[DefaultInfo].files.to_list()
        exe_runfiles = binary[DefaultInfo].default_runfiles
        runfiles = exe_runfiles if runfiles == None else runfiles.merge(exe_runfiles)
    else:
        binary_files = [binary]
    if executable == None:
        if len(binary_files) != 1:
            fail("There must be exactly one executable (got %s)" % binary_files)
        executable = binary_files[0]

    if type(args) == "Args":
        # You can't read args in bazel rules. So instead we write the args to a
        # file and read from that file at runtime.
        basename = out.basename.rsplit(".", 1)[0]

        # We could define a separate executable target, but that would mean that
        # users would need to add something like this attribute to their rule:
        # _write_to_file = attr.label(default=Label("//bazel/bash:write_to_file"))
        write_to_file = ctx.actions.declare_file(basename + "_write_to_file.sh")
        ctx.actions.write(write_to_file, _WRITE_TO_FILE, is_executable = True)
        args_file = ctx.actions.declare_file(basename + "_args.txt")
        ctx.actions.run(
            outputs = [args_file],
            executable = write_to_file,
            arguments = [args_file, args],
        )
        runfiles = runfiles.merge(ctx.runfiles(files = [args_file]))
        args = "$(cat %s)" % bash_rlocation(ctx, args_file)
    else:
        new_args = []
        for arg in args:
            if type(arg) == "File":
                new_args.append('"%s"' % (bash_rlocation(ctx, arg)))
            elif type(arg) == "string":
                new_args.append("'%s'" % (arg))
            else:
                fail("Unknown type '%s' for arg '%s'" % (type(arg), arg))
        args = " ".join(new_args)
    return generate_bash_script(
        ctx,
        out,
        content = '{content_prefix}\n\nexec "{binary}" {args} "$@"'.format(
            content_prefix = content_prefix,
            binary = bash_rlocation(ctx, executable),
            args = args,
        ),
        data = data,
        runfiles = runfiles,
    )

def _custom_args_binary_impl(ctx):
    if not ctx.attr.binary_args:
        fail("The binary_args attribute is required. If you used args, please instead use binary_args. Args attribute is reserved by bazel.")
    out = ctx.actions.declare_file(ctx.label.name + ".sh")
    return wrap_binary_with_args(
        ctx,
        out = out,
        binary = ctx.attr.binary,
        executable = ctx.executable.binary,
        args = ctx.attr.binary_args,
        data = ctx.attr.data,
    )

_CUSTOM_ARGS_ATTRS = dict(
    doc = """Generates a binary that runs another binary with a custom set of args.""",
    implementation = _custom_args_binary_impl,
    attrs = dict(
        binary = attr.label(executable = True, mandatory = True, cfg = "exec"),
        binary_args = attr.string_list(),
        **_COMMON_ATTRS
    ),
)
custom_args_binary = rule(executable = True, **_CUSTOM_ARGS_ATTRS)
custom_args_test = rule(test = True, **_CUSTOM_ARGS_ATTRS)
