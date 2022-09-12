// Copyright 2022 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package runfiles

import (
	"fmt"
	"os"
)

func FixEnv() {
	const (
		envRunfilesDir  = "RUNFILES_DIR"
		envManifestPath = "RUNFILES_MANIFEST_FILE"
	)

	if os.Getenv(envRunfilesDir) != "" || os.Getenv(envManifestPath) != "" {
		return
	}

	exe, err := os.Executable()
	if err != nil {
		panic(fmt.Sprintf("fixing environment variables for runfiles access: %v", err))
	}

	runfilesDir := exe + ".runfiles"
	manifestPath := exe + ".runfiles_manifest"

	ok := false
	if _, err := os.Stat(runfilesDir); err == nil {
		os.Setenv(envRunfilesDir, runfilesDir)
		ok = true
	}
	if _, err := os.Stat(manifestPath); err == nil {
		os.Setenv(envManifestPath, manifestPath)
		ok = true
	}

	if !ok {
		panic(fmt.Sprintf("failed to locate runfiles for %s", exe))
	}
}
