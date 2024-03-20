# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# TODO: Switch to distroless base image after cc toolchains are fully hermetic
FROM ubuntu:22.04
# TODO: Remove after cc toolchains are fully hermetic
RUN apt-get update && yes | apt-get install -y build-essential
