// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	_ "embed"
	"fmt"
	"log"
	"os"
	"os/exec"
	"os/signal"
	"path/filepath"
	"strings"

	"cros.local/bazel/ebuild/private/common/fileutil"
	"github.com/urfave/cli/v2"
	"golang.org/x/sys/unix"

	"cros.local/bazel/ebuild/private/common/bazelutil"
	"cros.local/bazel/ebuild/private/common/hostcontainercomms/host"
	"cros.local/bazel/ebuild/private/common/mountsdk"
)

var flagBoard = &cli.StringFlag{
	Name:     "board",
	Required: true,
}

var flagOutput = &cli.StringFlag{
	Name:     "output",
	Required: true,
}

var app = &cli.App{
	Flags: append(mountsdk.CLIFlags,
		flagBoard,
		mountsdk.FlagInstallTarget,
		flagOutput,
	),
	Action: func(c *cli.Context) error {
		board := c.String(flagBoard.Name)
		finalOutPath := c.String(flagOutput.Name)
		installTargetsUnparsed := c.StringSlice(mountsdk.FlagInstallTarget.Name)

		// We need "supports-graceful-termination" execution requirement in the
		// build action to let Bazel send SIGTERM instead of SIGKILL.
		ctx, cancel := signal.NotifyContext(c.Context, unix.SIGINT, unix.SIGTERM)
		defer cancel()
		c.Context = ctx

		cfg, err := mountsdk.GetMountConfigFromCLI(c)
		if err != nil {
			return err
		}

		targetPackagesDir := filepath.Join("/build", board, "packages")
		// AddInstallTargetsToConfig returns a set of environment variables for the
		// packages you want to install. We want to drop this to avoid calling
		// emerge on every package which we know is already installed.
		_, err = mountsdk.AddInstallTargetsToConfig(installTargetsUnparsed, targetPackagesDir, cfg)
		if err != nil {
			return err
		}

		if err := host.RunInSDKWithServer(ctx, cfg, func(s *mountsdk.MountedSDK) error {
			args := append([]string{
				// TODO: build_image has some exponential backoff for stuff like
				// mounting, which makes it impossible to debug because it never fails.
				// For now, we'll set a timeout which we'll remove later.
				"timeout",
				"60",
				filepath.Join(mountsdk.SourceDir, "chromite/bin/build_image"),
				fmt.Sprintf("--board=%s", board),
				// TODO: at some point, we should, instead of always building a test
				// image, take in some flags that allow us to choose the type of image
				//to build.
				"test",
			},
				c.Args().Slice()...)

			cmd := s.Command(c.Context, args[0], args[1:]...)
			cmd.Env = append(cmd.Env, fmt.Sprintf("BOARD=%s", board))
			if err := cmd.Run(); err != nil {
				return fmt.Errorf("Failed to run %s: %v", strings.Join(args, " "), err)
			}

			// TODO: get the path once we have a successful build.
			path := "/built_image"
			return fileutil.Copy(filepath.Join(s.DiffDir, path), finalOutPath)
		}); err != nil {
			if err, ok := err.(*exec.ExitError); ok {
				os.Exit(err.ExitCode())
			}
			return err
		}

		return nil
	},
}

func main() {
	bazelutil.FixRunfilesEnv()

	if err := app.Run(os.Args); err != nil {
		log.Fatalf("ERROR: %v", err)
	}
}
