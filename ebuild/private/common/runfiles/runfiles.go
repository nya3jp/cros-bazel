// Copyright 2022 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package runfiles

import (
	"fmt"
	"os"
)

func FixEnv() {
	const envName = "RUNFILES_DIR"
	if os.Getenv(envName) != "" {
		return
	}

	exe, err := os.Executable()
	if err != nil {
		panic(fmt.Sprintf("fixing environment variables for runfiles access: %v", err))
	}

	os.Setenv(envName, exe+".runfiles")
}
