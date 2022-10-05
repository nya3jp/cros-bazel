// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package ebuild

import _ "embed"

//go:embed ebuild_prelude.sh
var preludeCode []byte
