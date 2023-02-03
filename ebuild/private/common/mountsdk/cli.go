// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package mountsdk

import (
	"fmt"

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
		"Inside path must be absolute.",
}

var flagLogin = &cli.StringFlag{
	Name: "login",
	Usage: "--login=before|after|after-fail " +
		"logs in to the SDK before installing deps, before building, after building, or " +
		"after failing to build respectively.",
	Action: func(c *cli.Context, value string) error {
		mode := loginMode(value)
		if mode != loginNever && mode != loginBefore && mode != loginAfter && mode != loginAfterFail {
			return fmt.Errorf("invalid login mode: got %q; want one of %q, %q, or %q",
				mode, loginBefore, loginAfter, loginAfterFail)
		}
		return nil
	},
}

var CLIFlags = []cli.Flag{
	flagSDK,
	flagOverlay,
	flagLogin,
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
		cfg.Overlays = append(cfg.Overlays, overlay)
	}

	cfg.loginMode = loginMode(c.String(flagLogin.Name))
	return &cfg, nil
}
