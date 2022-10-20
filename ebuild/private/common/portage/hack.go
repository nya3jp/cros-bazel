package portage

// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

import (
	"strings"

	"cros.local/bazel/ebuild/private/common/standard/config"
	"cros.local/bazel/ebuild/private/common/standard/version"
)

// HACK: Hard-code several USE flags.
// TODO: Support USE_EXPAND and remove this hack.
var forceUse = []string{
	"board_use_arm64-generic",
	"chromeos_kernel_family_chromeos",
	"cpu_flags_arm_neon",
	"elibc_glibc",
	"input_devices_evdev",
	"kernel_linux",
	"linux_firmware_iwlwifi-all",
	"linux_firmware_rt2870",
	"linux_firmware_rtl8153",
	"ozone_platform_default_gbm",
	"ozone_platform_gbm",
	"ozone_platform_headless",
	"python_single_target_python3_6",
	"python_targets_python3_6",
	"ruby_targets_ruby25",
	"video_cards_llvmpipe",
}

// HACK: Hard-code several packages not to be installed.
var forceProvided = []string{
	// This package was used to force rust binary packages to rebuild.
	// We no longer need this workaround with bazel.
	"virtual/rust-binaries",

	// This is really a BDEPEND and there is no need to declare it as a
	// RDEPEND.
	"virtual/rust",
}

func NewHackSource() *config.HackSource {
	var providedPackages []*config.TargetPackage
	for _, name := range forceProvided {
		providedPackages = append(providedPackages, &config.TargetPackage{
			Name:    name,
			Version: &version.Version{Main: []string{"0"}}, // assume version 0
		})
	}
	return config.NewHackSource(strings.Join(forceUse, " "), providedPackages)
}
