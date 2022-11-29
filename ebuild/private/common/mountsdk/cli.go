// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package mountsdk

import (
	"path/filepath"

	"cros.local/bazel/ebuild/private/common/makechroot"
	"github.com/urfave/cli/v2"
)

var flagSDK = &cli.StringSliceFlag{
	Name:     "sdk",
	Required: true,
}

var flagOverlay = &cli.StringSliceFlag{
	Name:     "overlay",
	Required: true,
	Usage: "<inside path>=<squashfs file | directory | tar.*>: " +
		"Mounts the file or directory at the specified path. " +
		"Inside path can be absolute or relative to /mnt/host/source/.",
}

var CLIFlags = []cli.Flag{
	flagSDK,
	flagOverlay,
}

func GetMountConfigFromCLI(c *cli.Context) (*Config, error) {
	cfg := Config{}

	for _, sdk := range c.StringSlice(flagSDK.Name) {
		cfg.Overlays = append(cfg.Overlays, makechroot.OverlayInfo{
			ImagePath: sdk,
			MountDir:  "/",
		})
	}

	overlays, err := makechroot.ParseOverlaySpecs(c.StringSlice(flagOverlay.Name))
	if err != nil {
		return nil, err
	}
	for _, spec := range overlays {
		overlay := makechroot.OverlayInfo{
			ImagePath: spec.ImagePath,
			MountDir:  spec.MountDir,
		}
		if !filepath.IsAbs(overlay.MountDir) {
			overlay.MountDir = filepath.Join(SourceDir, overlay.MountDir)
		}
		cfg.Overlays = append(cfg.Overlays, overlay)
	}
	return &cfg, nil
}
