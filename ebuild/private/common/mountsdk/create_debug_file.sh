#!/bin/bash

OUT_FILE="$1"
shift

# Do this in two seperate writes to ensure that we don't expand bash variables
# in the first but we do in the second.
cat <<'EOF' > "${OUT_FILE}"
#!/bin/bash

# --- begin runfiles.bash initialization v2 ---
# Copy-pasted from the Bazel Bash runfiles library v2.
set -uo pipefail; f=bazel_tools/tools/bash/runfiles/runfiles.bash
source "${RUNFILES_DIR:-/dev/null}/$f" 2>/dev/null || \
source "$(grep -sm1 "^$f " "${RUNFILES_MANIFEST_FILE:-/dev/null}" | cut -f2- -d' ')" 2>/dev/null || \
source "$0.runfiles/$f" 2>/dev/null || \
source "$(grep -sm1 "^$f " "$0.runfiles_manifest" | cut -f2- -d' ')" 2>/dev/null || \
source "$(grep -sm1 "^$f " "$0.exe.runfiles_manifest" | cut -f2- -d' ')" 2>/dev/null || \
  { echo>&2 "ERROR: cannot find $f"; exit 1; }; f=; set -e
# --- end runfiles.bash initialization v2 ---

# Arguments passed in during build time are passed in relative to the execroot,
# which means all files passed in are relative paths starting with bazel-out/
# Thus, we cd to the directory in our working directory containing a bazel-out.
wd="$(pwd)"
cd "${wd%%/bazel-out/*}"

# The runfiles manifest file contains relative paths, which are evaluated
# relative to the working directory. Since we provide our own working directory,
# we need to use the RUNFILES_DIR instead.
export RUNFILES_DIR="${RUNFILES_MANIFEST_FILE%_manifest}"
unset RUNFILES_MANIFEST_FILE
EOF

# Unfortunately, this can't deal with characters such as ', ", and spaces in
# arguments. Fortunately, we don't appear to have any for now.
cat <<EOF >>  "${OUT_FILE}"
exec ${@@Q} "\$@"
EOF