// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"fmt"
	"os"
	"os/exec"

	"github.com/urfave/cli/v2"

	"cros.local/bazel/ebuild/private/common/cliutil"
	"cros.local/bazel/ebuild/private/common/symindex"
)

var flagInput = &cli.StringFlag{
	Name:     "input",
	Usage:    "A path to a .tar.xz archive file containing base SDK",
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
	Flags: []cli.Flag{
		flagInput,
		flagOutputDir,
		flagOutputSymindex,
	},
	Action: func(c *cli.Context) error {
		inputArchivePath := c.String(flagInput.Name)
		outputDirPath := c.String(flagOutputDir.Name)
		outputSymindexPath := c.String(flagOutputSymindex.Name)

		if err := os.MkdirAll(outputDirPath, 0o755); err != nil {
			return err
		}

		// TODO: Remove the dependency to the system pixz.
		cmd := exec.Command("/usr/bin/tar", "-I/usr/bin/pixz", "-xf", inputArchivePath, "-C", outputDirPath)
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		if err := cmd.Run(); err != nil {
			return fmt.Errorf("tar failed: %w", err)
		}

		if err := symindex.Generate(outputDirPath, outputSymindexPath); err != nil {
			return err
		}
		return nil
	},
}

func main() {
	cliutil.Exit(app.Run(os.Args))
}
