// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"log"
	"os"
	"runtime"

	"github.com/urfave/cli/v2"

	"cros.local/bazel/ebuild/private/cmd/fakefs/hooks"
	"cros.local/bazel/ebuild/private/cmd/fakefs/logging"
	"cros.local/bazel/ebuild/private/cmd/fakefs/tracee"
	"cros.local/bazel/ebuild/private/cmd/fakefs/tracer"
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
		flagCompatS,
		flagCompatI,
	},
	Action: func(c *cli.Context) error {
		runTracee := c.Bool(flagTracee.Name)
		verbose := c.Bool(flagVerbose.Name)
		if c.Args().Len() == 0 {
			cli.ShowAppHelpAndExit(c, 1)
		}

		hook := hooks.New()

		if runTracee {
			return tracee.Run(c.Args().Slice(), hook)
		}
		return tracer.Run(os.Args, hook, logging.New(verbose))
	},
}

func main() {
	// Lock the main thread to avoid confusing ptrace(2).
	runtime.LockOSThread()

	if err := app.Run(os.Args); err != nil {
		log.Fatalf("FATAL: %v", err)
	}
}
