// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package bazelutil

import "os"

// WorkspaceDir returns the path to the Bazel workspace directory, that is,
// the src directory in the current CrOS source checkout.
// This function returns an empty string if the program is not started by
// "bazel run".
func WorkspaceDir() string {
	return os.Getenv("BUILD_WORKSPACE_DIRECTORY")
}
