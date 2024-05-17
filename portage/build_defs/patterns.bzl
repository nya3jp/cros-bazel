# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# The list of files that are only needed when invoking emerge. We don't need
# these when building a package since alchemist has performed the analysis.
OVERLAY_ANALYSIS_FILE_PATTERN = [
    "profiles/**/bashrc/**",
    "profiles/**/eapi",
    "profiles/**/make.defaults",
    "profiles/**/package.accept_keywords",
    "profiles/**/package.accept_keywords/**",
    "profiles/**/package.bashrc",
    "profiles/**/package.bashrc/**",
    "profiles/**/package.keywords",
    "profiles/**/package.keywords/**",
    "profiles/**/package.mask",
    "profiles/**/package.mask/**",
    "profiles/**/package.provided",
    "profiles/**/package.provided/**",
    "profiles/**/package.unmask",
    "profiles/**/package.unmask/**",
    "profiles/**/package.use",
    "profiles/**/package.use/**",
    "profiles/**/package.use.force",
    "profiles/**/package.use.force/**",
    "profiles/**/package.use.mask",
    "profiles/**/package.use.mask/**",
    "profiles/**/package.use.stable.force",
    "profiles/**/package.use.stable.force/**",
    "profiles/**/package.use.stable.mask",
    "profiles/**/package.use.stable.mask/**",
    "profiles/**/packages",
    "profiles/**/parent",
    "profiles/**/profile.bashrc",
    "profiles/**/use.force",
    "profiles/**/use.force/**",
    "profiles/**/use.mask",
    "profiles/**/use.mask/**",
]

# Patterns matching files that aren't used for building packages.
NON_BUILD_FILE_PATTERNS = [
    "**/OWNERS",
    "**/README",
    "**/README.md",
]

# These files don't serve a purpose inside the container.
OVERLAY_EXCLUDE = NON_BUILD_FILE_PATTERNS + [
    "profiles/**/*.desc",
    "profiles/**/package.use.mask.example",
]
