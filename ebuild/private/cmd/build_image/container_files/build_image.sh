#!/bin/bash

# Some scripts call build_dlc, which is in the chromite bin directory.
export PATH="$PATH:/mnt/host/source/chromite/bin"

/mnt/host/source/chromite/bin/build_image "$@"
RC=$?

set -ex

# TODO: scripts/build_image.sh builds a base image, then copies it and modifies
# the copy to build dev / test images. We should convert base / dev / test
# images to three seperate bazel rules, so that changing a dev image package
# doesn't rebuild the whole base image.
if [ "${RC}" -eq 0 ]; then
  chown "${HOST_UID}:${HOST_GID}" /mnt/host/source/src/build/images/arm64-generic/latest/chromiumos_base_image.bin
fi

# TODO: remove temporary files owned by root, handle sigint / sigterm so that we still remove those files on error.
exit "${RC}"