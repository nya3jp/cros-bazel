# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# TODO: Switch to distroless base image after cc toolchains are hermetic
FROM ubuntu:22.04
# TODO: Remove build-essential after cc toolchains are hermetic
RUN apt-get update && \
    apt-get install -y --no-install-recommends build-essential sudo

# Allow all users to run sudo in the container. This is needed to mount
# privileges overlayfs.
COPY allow_sudo /etc/sudoers.d/
