#!/bin/bash -eu

# Update_crates starts up bazel with different settings, and hence requires
# bazel to restart. We can start it up with a different output_base to avoid
# this issue.
exec "${BUILD_WORKSPACE_DIRECTORY}/tools/bazel" --output_base=/tmp/bazel_rust "$@"