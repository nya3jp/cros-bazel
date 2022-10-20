// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package commonflags

import (
	"path/filepath"

	"github.com/urfave/cli"

	"cros.local/bazel/ebuild/private/common/bazelutil"
)

var DepsJSON = &cli.StringFlag{
	Name:  "deps-json",
	Value: filepath.Join(bazelutil.WorkspaceDir(), "bazel/data/deps.json"),
	Usage: "path to deps.json containing dependency graph data",
}

var DistfilesJSON = &cli.StringFlag{
	Name:  "distfiles-json",
	Value: filepath.Join(bazelutil.WorkspaceDir(), "bazel/data/distfiles.json"),
	Usage: "path to distfiles.json containing cached distfiles entries",
}
