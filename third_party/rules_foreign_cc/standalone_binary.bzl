load("@rules_foreign_cc//foreign_cc:providers.bzl", "ForeignCcDepsInfo")

# Note that I found that #!/bin/sh did not work.
# See https://stackoverflow.com/questions/59757296/ignore-failure-on-source-command-in-bash-script
BASH_FILE_CONTENTS = """#!/bin/bash

# --- begin runfiles.bash initialization v2 ---
# Copy-pasted from the Bazel Bash runfiles library v2.
set -uo pipefail; f=bazel_tools/tools/bash/runfiles/runfiles.bash
source "${RUNFILES_DIR:-/dev/null}/$f" 2>/dev/null || \\
source "$(grep -sm1 "^$f " "${RUNFILES_MANIFEST_FILE:-/dev/null}" | cut -f2- -d' ')" 2>/dev/null || \\
source "$0.runfiles/$f" 2>/dev/null || \\
source "$(grep -sm1 "^$f " "$0.runfiles_manifest" | cut -f2- -d' ')" 2>/dev/null || \\
source "$(grep -sm1 "^$f " "$0.exe.runfiles_manifest" | cut -f2- -d' ')" 2>/dev/null || \\
  { echo>&2 "ERROR: cannot find $f"; exit 1; }; f=; set -e
# --- end runfiles.bash initialization v2 ---

set -e

export LD_LIBRARY_PATH={EXTRA_LIBRARIES}:${LD_LIBRARY_PATH:-}

exec $(rlocation __main__/{BINARY_LOCATION}) "$@"
"""

def _foreign_cc_standalone_binary_impl(ctx):
    out = ctx.actions.declare_file(ctx.label.name)

    artifacts = ctx.attr.src[ForeignCcDepsInfo].artifacts.to_list()

    exe_files = ctx.attr.src[DefaultInfo].files.to_list()
    exe = None
    for f in exe_files:
        if f.basename == out.basename:
            exe = f

    if exe == None:
        fail("Unable to find binary '{bin}' in '{exe_files}'".format(
            bin = ctx.attr.bin_file,
            exe_files = exe_files,
        ))

    lib_paths = []
    deps = []
    for artifact in artifacts:
        deps.append(artifact.gen_dir)
        lib_paths.append("$(rlocation __main__/{artifact_dir}/{lib_dir})".format(
            artifact_dir = artifact.gen_dir.short_path,
            lib_dir = artifact.lib_dir_name,
        ))

    ctx.actions.write(
        output = out,
        content = BASH_FILE_CONTENTS
            .replace("{BINARY_LOCATION}", exe.short_path)
            .replace("{EXTRA_LIBRARIES}", ":".join(lib_paths)),
        is_executable = True,
    )

    runfiles = ctx.attr._bash_runfiles[DefaultInfo].files.to_list()
    return DefaultInfo(
        files = depset([out]),
        runfiles = ctx.runfiles(files = [exe] + deps + runfiles),
        executable = out,
    )

foreign_cc_standalone_binary = rule(
    _foreign_cc_standalone_binary_impl,
    attrs = dict(
        src = attr.label(mandatory = True, providers = [ForeignCcDepsInfo]),
        _bash_runfiles = attr.label(default = "@bazel_tools//tools/bash/runfiles"),
    ),
    doc = "Creates a binary that uses shared libraries provided by foreign_cc.",
    executable = True,
)
