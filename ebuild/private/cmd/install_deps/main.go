// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	_ "embed"
	"fmt"
	"os"
	"os/exec"
	"os/signal"
	"path/filepath"

	"github.com/urfave/cli/v2"
	"golang.org/x/sys/unix"

	"cros.local/bazel/ebuild/private/common/bazelutil"
	"cros.local/bazel/ebuild/private/common/cliutil"
	"cros.local/bazel/ebuild/private/common/fileutil"
	"cros.local/bazel/ebuild/private/common/mountsdk"
	"cros.local/bazel/ebuild/private/common/processes"
	"cros.local/bazel/ebuild/private/common/symindex"
)

const (
	binaryExt = ".tbz2"
)

//go:embed run.sh
var runScript []byte

var flagBoard = &cli.StringFlag{
	Name:     "board",
	Required: true,
}

var flagOutputDir = &cli.StringFlag{
	Name:     "output-dir",
	Usage:    "A path to a directory to write non-symlink files under",
	Required: true,
}

var flagOutputSymindex = &cli.StringFlag{
	Name:     "output-symindex",
	Usage:    "A path to write a symindex file to",
	Required: true,
}

var app = &cli.App{
	Flags: append(mountsdk.CLIFlags,
		flagBoard,
		mountsdk.FlagInstallTarget,
		flagOutputDir,
		flagOutputSymindex,
	),
	Action: func(c *cli.Context) error {
		outputDirPath := c.String(flagOutputDir.Name)
		outputSymindexPath := c.String(flagOutputSymindex.Name)
		board := c.String(flagBoard.Name)
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

		cfg.MainScript = runScript

		targetPackagesDir := filepath.Join("/build", board, "packages")

		installTargetsEnv, err := mountsdk.AddInstallTargetsToConfig(installTargetsUnparsed, targetPackagesDir, cfg)
		if err != nil {
			return err
		}

		if err := mountsdk.RunInSDK(cfg, func(s *mountsdk.MountedSDK) error {
			for _, dir := range []string{targetPackagesDir, "/var/lib/portage/pkgs"} {
				if err := os.MkdirAll(s.RootDir.Add(dir).Outside(), 0o755); err != nil {
					return err
				}
			}

			// TODO: Revisit MountedSDK's API to avoid passing an empty command here.
			cmd := s.Command("")
			cmd.Env = append(append(cmd.Env, installTargetsEnv...), fmt.Sprintf("BOARD=%s", board))

			if err := processes.Run(ctx, cmd); err != nil {
				return err
			}

			if err := fileutil.MoveDirContents(s.DiffDir, outputDirPath); err != nil {
				return err
			}

			if err := symindex.Generate(outputDirPath, outputSymindexPath); err != nil {
				return err
			}
			return nil
		}); err != nil {
			if err, ok := err.(*exec.ExitError); ok {
				return cliutil.ExitCode(err.ExitCode())
			}
			return err
		}

		return nil
	},
}

func main() {
	bazelutil.FixRunfilesEnv()
	cliutil.Exit(app.Run(os.Args))
}
