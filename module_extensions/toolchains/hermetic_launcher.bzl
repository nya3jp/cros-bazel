# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

load("//bazel/bash:defs.bzl", "BASH_RUNFILES_ATTRS")

visibility(["//bazel/cc", "//bazel/module_extensions/toolchains/..."])

# Use the same technique used by the toolchain SDK to make their binaries
# hermetic.
_WRAPPER_CONTENT = """#!/bin/bash
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

INTERP="$(rlocation _main~toolchains~toolchain_sdk/lib64/ld-linux-x86-64.so.2)"
LIBS="${INTERP%/lib64/ld-linux-x86-64.so.2}/lib"

SHELL_SCRIPT="$(realpath "${BASH_SOURCE[0]}")"
BIN="${SHELL_SCRIPT}.elf"
LD_ARGV0_REL="$(realpath --relative-to="${INTERP}" "${BIN}")"

# This *should* never happen for a well-configured target.
# If it does, we can solve it by creating three files instead of 2:
# * foo (symlink to foo.sh)
# * foo.sh (finds foo.elf from `realpath "${BASH_SOURCE[0]}`)
# * foo.elf
if [[ ! -f "${BIN}" ]]; then
    echo "Unable to find ${BIN}. Did you remember to pass the runfiles through transitively." >&2
    exit 1
fi

LD_ARGV0_REL="${LD_ARGV0_REL}" exec "${INTERP}" \
    --argv0 "$0" \
    --library-path "${LIBS}" \
    --inhibit-rpath '' \
    "${BIN}" \
    "$@"
"""

def hermetic_defaultinfo(ctx, files, runfiles, executable, symlink = False):
    out = ctx.actions.declare_file(ctx.label.name)
    extra = [out, executable]

    # Only actually use this script if we're using the hermetic toolchain.
    # Otherwise we just symlink this to the nonhermetic generated binary.
    if symlink:
        ctx.actions.symlink(output = out, target_file = executable)
    else:
        ctx.actions.write(
            out,
            _WRAPPER_CONTENT,
            is_executable = True,
        )
        extra.extend(ctx.files._libs)
        extra.append(ctx.file._bash_runfiles)
        extra.append(ctx.file._interp)
    extra_runfiles = ctx.runfiles(files = extra)
    if runfiles == None:
        runfiles = ctx.runfiles(files = extra)
    else:
        runfiles = runfiles.merge(ctx.runfiles(files = extra))
    return DefaultInfo(
        files = depset([out], transitive = [files]),
        runfiles = runfiles,
        executable = out,
    )

HERMETIC_ATTRS = dict(
    _interp = attr.label(default = "@@//bazel/module_extensions/toolchains/files:interp", allow_single_file = True),
    _libs = attr.label(default = "@@//bazel/module_extensions/toolchains/files:libs"),
) | BASH_RUNFILES_ATTRS
