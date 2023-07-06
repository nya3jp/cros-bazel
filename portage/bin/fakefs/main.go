// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"os"
	"runtime"

	"github.com/urfave/cli/v2"

	"cros.local/bazel/portage/bin/fakefs/tracee"
	"cros.local/bazel/portage/bin/fakefs/tracer"
	"cros.local/bazel/portage/common/cliutil"
)

var flagTracee = &cli.BoolFlag{
	Name:   "tracee",
	Usage:  "internal flag to start a tracee",
	Hidden: true,
}

var flagVerbose = &cli.BoolFlag{
	Name:    "verbose",
	Aliases: []string{"v"},
	Usage:   "enable verbose logging",
}

var flagPreload = &cli.StringFlag{
	Name:  "preload",
	Usage: "shared library to be added to LD_PRELOAD",
}

var flagCompatS = &cli.StringFlag{
	Name:   "s",
	Usage:  "for compatibility with fakeroot (ignored)",
	Hidden: true,
}

var flagCompatI = &cli.StringFlag{
	Name:   "i",
	Usage:  "for compatibility with fakeroot (ignored)",
	Hidden: true,
}

var app = &cli.App{
	Flags: []cli.Flag{
		flagTracee,
		flagVerbose,
		flagPreload,
		flagCompatS,
		flagCompatI,
	},
	Action: func(c *cli.Context) error {
		runTracee := c.Bool(flagTracee.Name)
		preloadPath := c.String(flagPreload.Name)
		verbose := c.Bool(flagVerbose.Name)
		args := c.Args().Slice()
		if len(args) == 0 {
			cli.ShowAppHelpAndExit(c, 1)
		}

		if runTracee {
			return tracee.Run(args)
		}
		return tracer.Run(os.Args, args, preloadPath, verbose)
	},
}

func main() {
	// Lock the main thread to avoid confusing ptrace(2).
	runtime.LockOSThread()
	cliutil.Exit(app.Run(os.Args))
}
