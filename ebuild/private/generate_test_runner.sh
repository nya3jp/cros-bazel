#!/bin/bash
# Copyright 2023 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

OUT_FILE="$1"
shift

# Do this in two seperate writes to ensure that we don't expand bash variables
# in the first but we do in the second.
cat <<'EOF' > "${OUT_FILE}"
#!/bin/bash

# Arguments passed in during build time are passed in relative to the execroot,
# which means all files passed in are relative paths starting with bazel-out/
# Thus, we cd to the directory in our working directory containing a bazel-out.
wd="$(pwd)"
cd "${wd%%/bazel-out/*}"

EOF

# Unfortunately, this can't deal with characters such as ', ", and spaces in
# arguments. Fortunately, we don't appear to have any for now.
cat <<EOF >>  "${OUT_FILE}"
exec ${@@Q} "\$@"
EOF
