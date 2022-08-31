// Copyright 2022 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package portagevars

import (
	"strings"

	"cros.local/bazel/ebuild/private/common/standard/makevars"
)

func Overlays(vars makevars.Vars) []string {
	return append([]string{vars["PORTDIR"]}, strings.Fields(vars["PORTDIR_OVERLAY"])...)
}
